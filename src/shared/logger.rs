use lazy_static::lazy_static;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    None,
    Error,
    Info,
    Debug,
}

lazy_static! {
    static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new(LogLevel::None));
}

#[derive(Debug, Clone)]
pub struct Logger {
    level: LogLevel,
}

impl Logger {
    fn new(level: LogLevel) -> Self {
        Self { level }
    }

    pub fn set_level(level: LogLevel) {
        if let Ok(mut logger) = LOGGER.lock() {
            *logger = Self::new(level);
        }
    }

    pub fn set_log_level(level: String) {
        let level = match level.to_lowercase().as_str() {
            "error" | "e" => LogLevel::Error,
            "info" | "i" => LogLevel::Info,
            "debug" | "d" => LogLevel::Debug,
            _ => LogLevel::None,
        };
        Logger::set_level(level);
    }

    // pub fn error(msg: &str) {
    //     if let Ok(logger) = LOGGER.lock() {
    //         if logger.level >= LogLevel::Error {
    //             println!("ERROR: {}", msg);
    //         }
    //     }
    // }

    pub fn info(msg: &str) {
        if let Ok(logger) = LOGGER.lock() {
            if logger.level >= LogLevel::Info {
                println!("INFO: {}", msg);
            }
        }
    }

    pub fn debug(msg: &str) {
        if let Ok(logger) = LOGGER.lock() {
            if logger.level >= LogLevel::Debug {
                println!("DEBUG: {}", msg);
            }
        }
    }

    // pub fn error_fmt(msg: &str, args: &[&dyn std::fmt::Debug]) {
    //     if let Ok(logger) = LOGGER.lock() {
    //         if logger.level >= LogLevel::Error {
    //             match args.len() {
    //                 0 => println!("ERROR: {}", msg),
    //                 1 => println!("ERROR: {}", format!("{} {:?}", msg, args[0])),
    //                 2 => println!("ERROR: {}", format!("{} {:?} {:?}", msg, args[0], args[1])),
    //                 _ => println!("ERROR: {}", msg),
    //             }
    //         }
    //     }
    // }

    pub fn info_fmt(msg: &str, args: &[&dyn std::fmt::Debug]) {
        if let Ok(logger) = LOGGER.lock() {
            if logger.level >= LogLevel::Info {
                match args.len() {
                    0 => println!("INFO: {}", msg),
                    1 => println!("INFO: {}", format!("{} {:?}", msg, args[0])),
                    2 => println!("INFO: {}", format!("{} {:?} {:?}", msg, args[0], args[1])),
                    _ => println!("INFO: {}", msg),
                }
            }
        }
    }

    pub fn debug_fmt(msg: &str, args: &[&dyn std::fmt::Debug]) {
        if let Ok(logger) = LOGGER.lock() {
            if logger.level >= LogLevel::Debug {
                match args.len() {
                    0 => println!("DEBUG: {}", msg),
                    1 => println!("DEBUG: {}", format!("{} {:?}", msg, args[0])),
                    2 => println!("DEBUG: {}", format!("{} {:?} {:?}", msg, args[0], args[1])),
                    _ => println!("DEBUG: {}", msg),
                }
            }
        }
    }
}
