use crate::common;
use chrono::prelude::*;
use lazy_static::lazy_static;
use std::io::Write;

lazy_static! {
    ///Config path pointing to default home
    pub static ref LOG_PATH: String = {
        let mut path = common::get_crypt_folder();
        path.push("logs");

        if !path.exists() {
            _ = std::fs::create_dir(&path);
        }

        let date = Local::now();

        let current_date = format!("{}-{}", date.year(), date.month());
        
        path.push(current_date);
        format!("{}", path.display())
    };
}


pub fn log_to_file(level: &str, message: &str) {
    let now = Local::now();
    let time = now.format("%d %H:%M:%S").to_string();

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_PATH.as_str())
        .unwrap();

    writeln!(file, "[{}] {}: {}", time, level, message).unwrap();
}

#[macro_export]
macro_rules! log {
    ($message:expr) => {
        crate::cli::logs::log_to_file("INFO", $message);
    };
}

#[macro_export]
macro_rules! error {
    ($message:expr) => {
        crate::cli::logs::log_to_file("ERROR", $message);
    };
}

pub use log;
pub use error;

