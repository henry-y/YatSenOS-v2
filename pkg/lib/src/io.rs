use crate::*;
use alloc::string::{String, ToString};
use alloc::vec;
// use ysos::logger;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    fn new() -> Self {
        Self
    }

    pub fn read_char(&self) -> Option<char> {
        let mut buf = vec![0; 4];
        // TODO: utf-8 support ?
        Stdout.write("begin read_char \n");
        if let Some(bytes) = sys_read(0, &mut buf) {
            Stdout.write("get bytes");
            Stdout.write(&bytes.to_string());
            Stdout.write("\n");
            if bytes > 0 {
                return Some(buf[0] as char);
            }
        }

        None
    }

    pub fn read_line(&self) -> String {
        // FIXME: allocate string
        let mut string = String::new();
        // FIXME: read from input buffer
        //       - maybe char by char?

        loop {
            // Stdout.write("begin read_line loop \n");
            if let Some(input) = self.read_char() {
                Stdout.write("input: \n ");
                Stdout.write(&input.to_string());
                Stdout.write("\n");
                match input {
                    '\n' => { Stdout.write("\n"); break; },
                    '\x08' => {
                        if string.len() > 0 {
                            string.pop();
                            Stdout.write("\x08 \x08");
                        }
                    },
                    '\x03' => {
                        // \x03 means Ctrl+C
                        string.clear();
                        break;
                    },
                    '\x04' => {
                        // \x04 means Ctrl+D
                        string.clear();
                        string.push('\x04');
                        break;
                    }
                    '\x00'..='\x1F' => {},
                    _ => {
                        string.push(input);
                        Stdout.write(&input.to_string());
                    }
                }


            }
        }
        // FIXME: handle backspace / enter...
        // FIXME: return string

        string
        
    }
}

impl Stdout {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(2, s.as_bytes());
    }
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}
