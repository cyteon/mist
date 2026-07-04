use chrono::Local;
use fancy_log::LogLevel;
use lazy_static::lazy_static;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::sync::Mutex;

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
}

pub fn log(level: LogLevel, message: &str) {
    fancy_log::log(level, message);

    // dont save debug logs
    if level != LogLevel::Debug {
        let timestamp = Local::now().format("%H:%M:%S");
        let log_message = format!("[{:?}] [{}] {}\n", level, timestamp, message);

        if let Ok(mut file) = LOG_FILE.lock() {
            file.write_all(log_message.as_bytes())
                .expect("Failed to write to log file");
        }
    }
}
