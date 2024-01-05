use crate::common;
use chrono::prelude::*;
use lazy_static::lazy_static;
use ansi_term::Color;
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

pub enum Level {
    Info,
    Warning,
    Error,
}

impl ToString for Level {
    fn to_string(&self) -> String {
        match self {
            Level::Info => String::from("INFO"),
            Level::Warning => String::from("WARNING"),
            Level::Error => String::from("ERROR"),
        }
    }
}

pub fn log_to_file(level: Level, path: &str, message: &str) {
    let now = Local::now();
    let time = now.format("%d %H:%M:%S").to_string();

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_PATH.as_str())
        .unwrap();

    writeln!(file, "[{} {}] {}: {}", time, path, level.to_string(), message).unwrap();

    let level_color = match level {
        Level::Info => Color::Green.bold(),
        Level::Warning => Color::Yellow.bold(),
        Level::Error => Color::Red.bold(),
    };

    println!("[{} {}] {}: {}", time, path, level_color.paint(level.to_string()).to_string(), message);
}

#[macro_export]
macro_rules! info {
    ($message:expr) => {
        crate::cli::logs::log_to_file(crate::cli::logs::Level::Info, module_path!(), $message);
    };
}

#[macro_export]
macro_rules! warning {
    ($message:expr) => {
        crate::cli::logs::log_to_file(crate::cli::logs::Level::Warning, module_path!(), $message);
    };
}

#[macro_export]
macro_rules! error {
    ($message:expr) => {
        crate::cli::logs::log_to_file(crate::cli::logs::Level::Error, module_path!(), $message);
    };
}

pub use info;
pub use warning;
pub use error;

