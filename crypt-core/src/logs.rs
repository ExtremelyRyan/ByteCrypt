use crate::{common, config};
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

        let current_date = format!("{}-{}.txt", date.year(), date.month());
        
        path.push(current_date);
        format!("{}", path.display())
    };
}

/// The type of information being parsed
/// # Types:
///```no_run
/// Info
/// Warning
/// Error
/// Critical
///```
pub enum Level {
    /// Messages sent or stored for relaying basic information
    Info,
    /// Messages sent or stored relaying warnings within the system
    Warning,
    /// Messages sent or stored relaying errors within the system
    Error,
    /// Messages sent or stored relaying catastrophic errors within the system
    Critical,
}

impl ToString for Level {
    fn to_string(&self) -> String {
        match self {
            Level::Info => String::from("INFO"),
            Level::Warning => String::from("WARNING"),
            Level::Error => String::from("ERROR"),
            Level::Critical => String::from("CRITICAL"),
        }
    }
}

pub fn log(level: Level, path: &str, message: &str) {
    let now = Local::now();
    let time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_PATH.as_str())
        .unwrap();

    writeln!(file, "[{} {}] {}: {}", time, path, level.to_string(), message).unwrap();

    match config::get_interface() {
        config::Interface::CLI => {
            let level_color = match level {
                Level::Info => Color::Green.normal(),
                Level::Warning => Color::Yellow.normal(),
                Level::Error => Color::Red.normal(),
                Level::Critical => Color::Red.bold(),
            };

            println!("[{} {}] {}: {}", 
                time, 
                path, 
                level_color.paint(level.to_string()).to_string(), 
                message
            );
        },
        _ => (),
    }
}

#[macro_export]
macro_rules! info {
    ($message:expr) => {
        log(Level::Info, module_path!(), $message);
    };
}

#[macro_export]
macro_rules! warning {
    ($message:expr) => {
        log(Level::Warning, module_path!(), $message);
    };
}

#[macro_export]
macro_rules! error {
    ($message:expr) => {
        log(Level::Error, module_path!(), $message);
    };
}

#[macro_export]
macro_rules! critical {
    ($message:expr) => {
        log(Level::Critical, module_path!(), $message);
    };
}

pub use info;
pub use warning;
pub use error;
pub use critical;

