use nix::unistd;
use std::ffi::CString;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let stdin = std::io::stdin();
    let mut line = String::new();
    let username = unistd::User::from_uid(unistd::getuid())
        .unwrap()
        .unwrap()
        .name;
    let hostname = unistd::gethostname().unwrap().into_string().unwrap();
    loop {
        print!("[{}@{}]$ ", username, hostname);
        std::io::stdout().flush().unwrap();
        line.clear();
        stdin.read_line(&mut line).unwrap();
        let mut line_it = line.split(" ");
        let exec = line_it.next().unwrap();
        let absolute_exec = std::env::var("PATH")
            .unwrap()
            .split(':')
            .map(PathBuf::from)
            .find_map(|mut path| {
                path.push(exec.trim());
                if path.is_file() { Some(path) } else { None }
            })
            .unwrap();
        let fork = unsafe { unistd::fork().unwrap() };
        match fork {
            unistd::ForkResult::Child => {
                let c_path = CString::new(absolute_exec.as_os_str().as_encoded_bytes()).unwrap();
                unistd::execv(&c_path, &[&c_path]).unwrap();
            }
            unistd::ForkResult::Parent { child: _, .. } => {
                nix::sys::wait::wait().unwrap();
            }
        }
    }
}
