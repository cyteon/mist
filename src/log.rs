use chrono::Local;
use lazy_static::lazy_static;
use rustyline::ExternalPrinter;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

lazy_static! {
    static ref LOG_FILE: Mutex<std::fs::File> = {
        create_dir_all("logs").expect("Failed to create logs directory");

        let file_name = format!("logs/mist-{}.log", Local::now().format("%Y-%m-%d"));

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_name)
            .expect("Failed to open log file");

        Mutex::new(file)
    };
    static ref PRINTER: Mutex<Option<Box<dyn ExternalPrinter + Send>>> = Mutex::new(None);
}

pub fn set_printer<P: ExternalPrinter + Send + 'static>(printer: P) {
    *PRINTER.lock().unwrap() = Some(Box::new(printer));
}

impl LogLevel {
    pub fn color(&self) -> &'static str {
        match self {
            LogLevel::Debug => "\x1b[1;90m",
            LogLevel::Info => "\x1b[1;32m",
            LogLevel::Warn => "\x1b[1;33m",
            LogLevel::Error => "\x1b[1;31m",
        }
    }

    fn as_u8(&self) -> u8 {
        match self {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        }
    }
}

static LOG_LEVEL: AtomicU8 = AtomicU8::new(1);

pub fn set_log_level(level: LogLevel) {
    LOG_LEVEL.store(level.as_u8(), Relaxed);
}

pub fn log(level: LogLevel, message: &str) {
    let timestamp = Local::now().format("%H:%M:%S");
    let line = format!(
        "{}[{:?}]\x1b[0m [{}] {}",
        level.color(),
        level,
        timestamp,
        message
    );

    let mut guard = PRINTER.lock().unwrap();
    if let Some(printer) = guard.as_mut() {
        printer.print(line);
    } else {
        println!("{}", line);
    }

    // dont save debug logs
    if level != LogLevel::Debug {
        let log_message = format!("[{:?}] [{}] {}\n", level, timestamp, message);

        if let Ok(mut file) = LOG_FILE.lock() {
            file.write_all(log_message.as_bytes())
                .expect("Failed to write to log file");
        }
    }
}
