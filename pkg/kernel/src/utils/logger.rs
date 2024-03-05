use log::{Metadata, Record};

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();

    // FIXME: Configure the logger

    log::set_max_level(log::LevelFilter::Trace);

    info!("Logger Initialized.");
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= metadata.level()
    }

    fn log(&self, record: &Record) {
        // FIXME: Implement the logger with serial output
        const RESET: &str = "\x1B[0m";
        const RED: &str = "\x1B[31m";
        const GREEN: &str = "\x1B[32m";
        const YELLOW: &str = "\x1B[33m";
        if self.enabled(record.metadata()) {
            if record.level() != log::Level::Info {
                print!("occur at {}:{} ", record.file_static().unwrap(), record.line().unwrap());   
            } 
            match record.level() {
                log::Level::Error => println!("{}[{}]{} {}", RED, record.level(), RESET, record.args()),
                log::Level::Warn => println!("{}[{}]{} {}", YELLOW, record.level(), RESET, record.args()),
                log::Level::Info => println!("{}[{}]{} {}", GREEN, record.level(), RESET, record.args()),
                log::Level::Trace => println!("{}[{}] {} {}", YELLOW, record.level(), record.args(), RESET),
                _ => println!("[{}] {}", record.level(), record.args()),
            }
        }
        
    }

    fn flush(&self) {}
}
