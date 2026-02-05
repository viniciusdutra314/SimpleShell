use std::ffi::{CStr, CString};
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;

fn main() {
    let stdin = std::io::stdin();
    let username = unsafe {
        let login = libc::getlogin();
        if login.is_null() {
            CString::new("username_not_found").unwrap()
        } else {
            CStr::from_ptr(login).to_owned()
        }
    };

    const HOSTNAME_BUFFER_SIZE: usize = 256;
    let mut hostname_buf = vec![0u8; HOSTNAME_BUFFER_SIZE];
    let ret = unsafe {
        libc::gethostname(
            hostname_buf.as_mut_ptr() as *mut libc::c_char,
            hostname_buf.len(),
        )
    };
    let hostname = if ret == 0 {
        let len = hostname_buf
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(hostname_buf.len());
        CString::new(&hostname_buf[..len]).unwrap()
    } else {
        CString::new("hostname_not_found").unwrap()
    };

    loop {
        print!(
            "(SimpleShell) [{}@{}]$ ",
            username.to_string_lossy(),
            hostname.to_string_lossy()
        );
        std::io::stdout().flush().unwrap();

        let mut line = String::new();
        if stdin.read_line(&mut line).unwrap() == 0 {
            println!();
            break;
        }

        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        let mut parts = trimmed_line.split_whitespace();
        let command = parts.next().unwrap();

        match command {
            "exit" => return (),
            "cd" => {
                if let Some(path) = parts.next() {
                    let path_c = CString::new(path).unwrap();
                    let path_pathbuf = PathBuf::from(path);
                    if path_pathbuf.is_dir() {
                        unsafe { libc::chdir(path_c.as_ptr()) };
                    } else {
                        println!("No such file or directory {}", path);
                    }
                }
            }
            _ => {
                let absolute_exec = std::env::var("PATH")
                    .unwrap()
                    .split(':')
                    .map(PathBuf::from)
                    .find_map(|mut path| {
                        path.push(command);
                        if path.is_file() { Some(path) } else { None }
                    });

                let exec_path = match absolute_exec {
                    Some(path) => path,
                    None => {
                        eprintln!("{}: command not found", command);
                        continue;
                    }
                };

                unsafe {
                    let pid = libc::fork();
                    if pid == 0 {
                        let c_path = CString::new(exec_path.as_os_str().as_bytes()).unwrap();
                        let mut c_args: Vec<CString> =
                            parts.map(|arg| CString::new(arg).unwrap()).collect();
                        c_args.insert(0, c_path.clone());
                        let mut argv: Vec<*const libc::c_char> =
                            c_args.iter().map(|s| s.as_ptr()).collect();
                        argv.push(ptr::null());
                        libc::execv(c_path.as_ptr(), argv.as_ptr());
                    } else if pid > 0 {
                        let mut status: libc::c_int = 0;
                        libc::waitpid(pid, &mut status, 0);
                    } else {
                        eprintln!("fork failed");
                    }
                }
            }
        }
    }
}
