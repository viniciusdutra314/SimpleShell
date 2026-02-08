
pub struct TermiosContext {
    initial_context: libc::termios,
}
impl TermiosContext {
    pub fn new() -> Option<Self> {
        unsafe {
            let mut initial_context = std::mem::MaybeUninit::uninit();
            if libc::tcgetattr(libc::STDIN_FILENO, initial_context.as_mut_ptr()) != 0 {
                return None;
            }
            return Some(Self {
                initial_context: initial_context.assume_init(),
            });
        }
    }
    pub fn get_initial(&self) -> libc::termios {
        return self.initial_context;
    }

    pub fn set(new_termios: &mut libc::termios) -> bool {
        unsafe {
            return libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, new_termios) != 0;
        }
    }
}

impl Drop for TermiosContext {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &self.initial_context);
        }
    }
}
