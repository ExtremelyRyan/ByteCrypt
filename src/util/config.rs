use anyhow::{Error, Ok, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::{fs, io::ErrorKind, path::Path};
use toml::*;

const CONFIG_PATH: &str = "config.toml";

#[derive(Deserialize, Serialize, Debug)]
///Holds the configuration for the program
pub struct Config {
    /// collection of cloud services currently holding crypt files.
    /// pub cloud_services: Vec<String>,
    /// serves as the default location for the SQLite database path.
    #[serde(default)]
    pub database_path: String,
    /// collection of any directories to ignore during folder encryption.
    #[serde(default)]
    pub ignore_directories: Vec<String>,
    /// option to retain both the original file after encryption,
    /// as well as the .crypt file after decryption.
    /// if true, retains original file and encrypted file.
    /// if false, deletes files after encryption / decryption.
    #[serde(default)]
    pub retain: bool,
    /// zstd level is for file compression, from [fastest, least compression]
    /// to [slowest, highest compression] `-7 - 22`. Default compression level is 3.
    #[serde(default = "default_zstd")]
    pub zstd_level: i32,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // _ = writeln!(f, "cloud_services: {:?}", self.cloud_services);
        _ = writeln!(f, "database_path: {}", self.database_path);
        _ = writeln!(f, "ignore_directories: {:?}", self.ignore_directories);
        _ = writeln!(f, "retain: {}", self.retain);
        _ = writeln!(f, "zstd_level: {}", self.zstd_level);
        std::fmt::Result::Ok(())
    }
}

fn default_zstd() -> i32 {
    3
}

///Default configuration
impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: "crypt_keeper.db".to_string(),
            // cloud_services: Vec::new(),
            retain: true,
            ignore_directories: vec![".".to_string()],
            zstd_level: 3,
        }
    }
}

impl Config {
    //Should I be returning anyhow error handling things?
    fn new(
        database_path: String,
        // cloud_services: Vec<String>,
        retain: bool,
        ignore_directories: Vec<String>,
        zstd_level: i32,
    ) -> Self {
        Self {
            database_path,
            // cloud_services,
            retain,
            ignore_directories,
            zstd_level,
        }
    }

    pub fn get_fields() -> Vec<&'static str> {
        vec![
            "database_path",
            "cloud_services",
            "retain",
            "hidden_directories",
        ]
    }

    ///Changes the database path
    pub fn change_db_path(&mut self, path: String) {
        self.database_path = path;
    }

    ///Adds a cloud service to the list
    // pub fn add_cloud_service(&mut self, service: String) {
    //     self.cloud_services.push(service);
    // }

    ///Removes a cloud service from the list
    // pub fn remove_cloud_service(&mut self, service: String) {
    //     self.cloud_services.retain(|s| s != &service);
    // }

    pub fn get_database_path(&self) -> &str {
        self.database_path.as_ref()
    }
    pub fn set_database_path(&mut self, path: &String) {
        self.database_path = path.to_owned();
        _ = save_config(self);
    }

    pub fn retain(&self) -> bool {
        self.retain
    }

    pub fn set_retain(&mut self, retain: String) -> bool {
        match retain.to_lowercase().as_str() {
            "true" | "t" => self.retain = true,
            "false" | "f" => self.retain = false,
            _ => return false,
        }
        if save_config(self).is_err() {
            return false;
        }
        true
    }

    pub fn ignore_directories(&self) -> &[String] {
        self.ignore_directories.as_ref()
    }

    pub fn set_ignore_directories(&mut self, ignore_directories: Vec<String>) {
        self.ignore_directories = ignore_directories;
        _ = save_config(self);
    }
    pub fn append_ignore_directories(&mut self, item: &String) {
        self.ignore_directories.push(item.to_owned());
        _ = save_config(self);
    }

    pub fn remove_item_from_ignore_directories(&mut self, item: &String) {
        if self.ignore_directories.contains(item) {
            let index = &self.ignore_directories.iter().position(|x| x == item);
            let num = index.unwrap();
            self.ignore_directories.remove(num);
            _ = save_config(self);
        }
    }

    pub fn get_zstd_level(&self) -> i32 {
        self.zstd_level
    }
    pub fn set_zstd_level(&mut self, level: i32) -> bool {
        match level {
            -7..=22 => {
                self.zstd_level = level;
                _ = save_config(self);
                true
            }
            _ => {
                println!("Error: invalid compression level. Please enter a number from -7 - 22");
                false
            }
        }
    }
}

///Loads configuration file -- creates default if missing
pub fn load_config() -> Result<Config> {
    info!("loading config");
    //If the file doesn't exist, re-create and load defaults
    if !Path::new(CONFIG_PATH).exists() {
        warn!("No configuration found, reloading with defaults!");
        let config = Config::default();
        save_config(&config)?;
    }

    //Load the configuration file from stored json
    let config_file = fs::read_to_string(CONFIG_PATH)?;
    let config = toml::from_str(&config_file)
        .map_err(|e| error!("error: {e}"))
        .unwrap();
    Ok(config)
}

///Saves the configuration file
pub fn save_config(config: &Config) -> Result<()> {
    info!("saving config");
    //Serialize config
    let serialized_config = toml::to_string_pretty(&config)?;
    Ok(fs::write(CONFIG_PATH, serialized_config)?)
}
