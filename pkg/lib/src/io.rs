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
        let mut buf = [0; 4];
        // TODO: utf-8 support ?
        // stdout().write("begin read_char \n");
        if let Some(bytes) = sys_read(0, &mut buf) {
            // stdout().write("get bytes");
            // stdout().write(&bytes.to_string());
            // stdout().write("\n");
            if bytes > 0 {
                // crate::println!("bytes: {}", buf[0]);
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
            // stdout().write("begin read_line loop \n");
            if let Some(input) = self.read_char() {
                // stdout().write("input: \n ");
                // stdout().write(&input.to_string());
                // stdout().write("\n");
                // crate::print!("{}", input);
                // crate::println!("input: {:x}", input as u16);
                // if input as u16 == 0xd {
                //     crate::println!("match enter succ .. ");
                //     stdout().write("\n");
                //     break;
                // }
                
                match input {
                    '\n' | '\r' => { 
                        // crate::println!("match enter succ .. ");
                        stdout().write("\n"); 
                        break; 
                    },
                    '\x08' | '\x7f' => {
                        // crate::println!("exec backspace .. ");
                        if string.len() > 0 {
                            string.pop();
                            stdout().write("\x08 \x08");
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
                        crate::print!("{}", input);
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
