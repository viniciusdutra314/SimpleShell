#![deny(clippy::disallowed_types)]

use std::ffi::CString;
use std::io::Write;

use crate::termios::TermiosContext;
mod posix_wrappers;
mod prefix_tree;
mod termios;

fn main() {
    let termios_context = termios::TermiosContext::new().unwrap();
    let mut new_context = termios_context.get_initial();
    unsafe {
        libc::cfmakeraw(&mut new_context);
    };
    TermiosContext::set(&mut new_context);

    loop {
        print!(
            "(SimpleShell) [{}@{}]$ ",
            posix_wrappers::get_username().unwrap(),
            posix_wrappers::get_hostname().unwrap(),
        );
        std::io::stdout().flush().unwrap();

        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).unwrap() == 0 {
            println!();
            break;
        }

        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        let mut split_iter = trimmed_line.split_whitespace();
        let command = split_iter.next().unwrap();

        match command {
            "exit" => return (),
            "cd" => {
                if let Some(path_str) = split_iter.next() {
                    let path = posix_wrappers::PosixPath::try_from(path_str).unwrap();
                    if !posix_wrappers::chdir(&path) {
                        println!("No such file or directory {}", path_str);
                    }
                } else {
                    if let Ok(home_dir) = std::env::var("HOME") {
                        let path = posix_wrappers::PosixPath::try_from(home_dir.as_str()).unwrap();
                        if !posix_wrappers::chdir(&path) {
                            println!("Failed to change to HOME directory");
                        }
                    } else {
                        println!("HOME environment variable not set");
                    }
                }
            }

            _ => {
                let exec_path = posix_wrappers::find_binary_using_path(command).unwrap();
                let arguments: Vec<CString> =
                    split_iter.map(|s| CString::new(s).unwrap()).collect();
                posix_wrappers::fork_and_execve(&exec_path, &arguments).unwrap();
            }
        }
    }
}
