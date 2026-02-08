use std::ffi::{CStr, CString};
use std::ops::Deref;
#[repr(transparent)]
pub struct PosixPath {
    buffer: CString,
}

impl PosixPath {
    pub fn is_dir(&self) -> bool {
        let mut buffer_uninit = std::mem::MaybeUninit::<libc::stat>::uninit();
        unsafe { libc::stat(self.buffer.as_ptr(), buffer_uninit.as_mut_ptr()) };
        let buffer = unsafe { buffer_uninit.assume_init() };
        return (buffer.st_mode & libc::S_IFMT) == libc::S_IFDIR;
    }

    fn validate_path(directory: &CStr) -> bool {
        let mut buffer_uninit = std::mem::MaybeUninit::<libc::stat>::uninit();
        return unsafe { libc::stat(directory.as_ptr(), buffer_uninit.as_mut_ptr()) } == 0;
    }
}
#[derive(Debug)]
pub struct InvalidPathError;
impl TryFrom<&CStr> for PosixPath {
    type Error = InvalidPathError;

    fn try_from(s: &CStr) -> Result<Self, Self::Error> {
        if PosixPath::validate_path(s) {
            Ok(PosixPath {
                buffer: s.to_owned(),
            })
        } else {
            Err(InvalidPathError)
        }
    }
}

impl TryFrom<&str> for PosixPath {
    type Error = InvalidPathError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let c_string = CString::new(s).map_err(|_| InvalidPathError)?;
        if PosixPath::validate_path(&c_string) {
            Ok(PosixPath { buffer: c_string })
        } else {
            Err(InvalidPathError)
        }
    }
}

impl Deref for PosixPath {
    type Target = CString;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

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

pub fn chdir(path: &PosixPath) -> bool {
    if path.is_dir() {
        return unsafe { libc::chdir(path.as_ptr()) == 0 };
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
