use lazy_static::lazy_static;
use std::fs::File;
use std::io::Write;
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
    static ref LOG_FILE: Mutex<Option<File>> = Mutex::new(None);
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

    /// Redirect all log output to a file instead of stdout.
    /// Call this before starting the TUI to avoid corrupting the display.
    pub fn set_log_file(path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        if let Ok(mut log_file) = LOG_FILE.lock() {
            *log_file = Some(file);
        }
        Ok(())
    }

    pub fn info_fmt(msg: &str, args: &[&dyn std::fmt::Debug]) {
        if let Ok(logger) = LOGGER.lock() {
            if logger.level >= LogLevel::Info {
                let line = match args.len() {
                    0 => format!("INFO: {}\n", msg),
                    1 => format!("INFO: {} {:?}\n", msg, args[0]),
                    2 => format!("INFO: {} {:?} {:?}\n", msg, args[0], args[1]),
                    _ => format!("INFO: {}\n", msg),
                };
                log_output(&line);
            }
        }
    }

    pub fn debug(msg: &str) {
        if let Ok(logger) = LOGGER.lock() {
            if logger.level >= LogLevel::Debug {
                log_output(&format!("DEBUG: {}\n", msg));
            }
        }
    }

    pub fn debug_fmt(msg: &str, args: &[&dyn std::fmt::Debug]) {
        if let Ok(logger) = LOGGER.lock() {
            if logger.level >= LogLevel::Debug {
                let line = match args.len() {
                    0 => format!("DEBUG: {}\n", msg),
                    1 => format!("DEBUG: {} {:?}\n", msg, args[0]),
                    2 => format!("DEBUG: {} {:?} {:?}\n", msg, args[0], args[1]),
                    _ => format!("DEBUG: {}\n", msg),
                };
                log_output(&line);
            }
        }
    }
}

fn log_output(line: &str) {
    if let Ok(mut file_opt) = LOG_FILE.lock() {
        if let Some(ref mut file) = *file_opt {
            let _ = file.write_all(line.as_bytes());
            return;
        }
    }
    print!("{}", line);
}
