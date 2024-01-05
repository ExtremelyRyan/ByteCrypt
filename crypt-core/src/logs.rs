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

pub fn log(message: &str) {
    let now = Local::now();
    let time = now.format("%d %H:%M:%S").to_string();

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_PATH.as_str())
        .unwrap();

    writeln!(file, "[{}] INFO: {}", time, message).unwrap();
}

pub fn error(message: &str) {
    let now = Local::now();
    let time = now.format("%d %H:%M:%S").to_string();

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_PATH.as_str())
        .unwrap();

    writeln!(file, "[{}] ERROR: {}", time, message).unwrap();
}
