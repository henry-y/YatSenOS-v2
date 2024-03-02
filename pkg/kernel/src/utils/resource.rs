use alloc::string::String;
use crate::{drivers::serial, drivers::input::try_pop_key};

#[derive(Debug, Clone)]
pub enum StdIO {
    Stdin,
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub enum Resource {
    Console(StdIO),
    Null,
}

impl Resource {
    pub fn read(&self, buf: &mut [u8]) -> Option<usize> {
        match self {
            Resource::Console(stdio) => match stdio {
                &StdIO::Stdin => {
                    // FIXME: just read from kernel input buffer
                    if let Some(k) = try_pop_key() {
                        buf[0] = k;
                        Some(1)
                    } else {
                        Some(0)
                    }
                }
                _ => None,
            },
            Resource::Null => Some(0),
        }
    }

    pub fn write(&self, buf: &[u8]) -> Option<usize> {
        match self {
            Resource::Console(stdio) => match *stdio {
                StdIO::Stdin => None,
                StdIO::Stdout => {
                    print!("{}", String::from_utf8_lossy(buf));
                    Some(buf.len())
                }
                StdIO::Stderr => {
                    warn!("{}", String::from_utf8_lossy(buf));
                    Some(buf.len())
                }
            },
            Resource::Null => Some(buf.len()),
        }
    }
}
