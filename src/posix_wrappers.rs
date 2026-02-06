use std::{
    ffi::{CStr, CString},
    path::Path,
};

pub fn get_username() -> Option<String> {
    unsafe {
        let login = libc::getlogin();
        if login.is_null() {
            None
        } else {
            Some(CStr::from_ptr(login).to_string_lossy().into_owned())
        }
    }
}
pub fn get_hostname() -> Option<String> {
    const HOSTNAME_BUFFER_SIZE: usize = 1024;
    let mut hostname_buf = vec![0u8; HOSTNAME_BUFFER_SIZE];
    let ret = unsafe {
        libc::gethostname(
            hostname_buf.as_mut_ptr() as *mut libc::c_char,
            hostname_buf.len(),
        )
    };

    if ret == 0 {
        let len = hostname_buf
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(hostname_buf.len());
        hostname_buf.truncate(len);
        String::from_utf8(hostname_buf).ok()
    } else {
        None
    }
}

pub fn chdir(path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    if path.is_dir() {
        if let Ok(path_c) = std::ffi::CString::new(path.as_os_str().as_bytes()) {
            return unsafe { libc::chdir(path_c.as_ptr()) == 0 };
        }
    }
    false
}
pub fn fork_and_execve(binary_path: &CStr, arguments: &[CString]) -> Result<(), std::io::Error> {
    unsafe {
        let pid = libc::fork();
        if pid < 0 {
            return Err(std::io::Error::last_os_error());
        }
        if pid == 0 {
            let mut argv: Vec<*const libc::c_char> = Vec::with_capacity(arguments.len() + 2);
            argv.push(binary_path.as_ptr());
            argv.extend(arguments.iter().map(|s| s.as_ptr()));
            argv.push(std::ptr::null());

            libc::execv(binary_path.as_ptr(), argv.as_ptr());
            eprintln!("execv failed: {}", std::io::Error::last_os_error());
            libc::exit(1);
        } else {
            let mut status: libc::c_int = 0;
            if libc::waitpid(pid, &mut status, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }
    Ok(())
}
pub fn find_binary_using_path(command: &str) -> Option<CString> {
    if command.contains('/') {
        // If command contains a slash, it's a direct path.
        // Check if it exists and is executable.
        let c_command = CString::new(command).ok()?;
        unsafe {
            if libc::access(c_command.as_ptr(), libc::X_OK) == 0 {
                return Some(c_command);
            }
        }
        return None;
    }

    let path_var = match std::env::var("PATH") {
        Ok(val) => val,
        Err(_) => return None,
    };

    for dir in path_var.split(':') {
        let full_path_str = format!("{}/{}", dir, command);
        if let Ok(c_path) = CString::new(full_path_str) {
            unsafe {
                if libc::access(c_path.as_ptr(), libc::X_OK) == 0 {
                    return Some(c_path);
                }
            }
        }
    }

    None
}
