use anyhow::anyhow;
use log::{error, info, warn};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    sync::Mutex,
    collections::HashMap,
    fs,
    io::{self, BufRead, BufReader, ErrorKind, Write},
    path::Path,
};
use toml::*;

const CONFIG_PATH: &str = "config.toml";

lazy_static! {
    static ref CONFIG: Mutex<Config> = Mutex::new({
        match load_config() {
            Ok(config) => config,
            Err(err) => panic!("Failed to load config: {}", err),
        }
    });
}

pub fn get_config() -> std::sync::MutexGuard<'static, Config> {
    CONFIG.lock().expect("Config is currently in lock")
}

#[derive(Deserialize, Serialize, Debug)]
///Holds the configuration for the program
pub struct Config {
    /// collection of cloud services currently holding crypt files.
    /// pub cloud_services: Vec<String>,
    /// serves as the default location for the SQLite database path.
    pub database_path: String,

    /// collection of any directories to ignore during folder encryption.
    pub ignore_items: Vec<String>,

    /// option to retain both the original file after encryption,
    /// as well as the .crypt file after decryption.
    /// if true, retains original file and encrypted file.
    /// if false, deletes files after encryption / decryption.
    pub retain: bool,

    /// zstd level is for file compression, from [fastest, least compression]
    /// to [slowest, highest compression] `-7 to 22`. Default compression level is 3.
    pub zstd_level: i32,
}

///Standard format for both CLI and TUI display
//TODO: Update for TUI purposes when completed
impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // _ = writeln!(f, "cloud_services: {:?}", self.cloud_services);
        _ = writeln!(f, " database_path: {}", self.database_path);
        _ = writeln!(f, " ignore_directories: {:?}", self.ignore_items);
        _ = writeln!(f, " retain: {}", self.retain);
        _ = writeln!(f, " zstd_level: {}", self.zstd_level);
        std::fmt::Result::Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database_path: "crypt_keeper.db".to_string(),
            // cloud_services: Vec::new(),
            retain: true,
            ignore_items: vec![".".to_string()],
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
            ignore_items: ignore_directories,
            zstd_level,
        }
    }

    pub fn get_fields() -> Vec<&'static str> {
        vec![
            "database_path",
            // "cloud_services",
            "ignore_directories",
            "retain",
            "zstd_level",
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

    pub fn ignore_items(&self) -> &[String] {
        self.ignore_items.as_ref()
    }

    pub fn set_ignore_items(&mut self, ignore_directories: Vec<String>) {
        self.ignore_items = ignore_directories;
        _ = save_config(self);
    }
    pub fn append_ignore_items(&mut self, item: &String) {
        self.ignore_items.push(item.to_owned());
        _ = save_config(self);
    }

    pub fn remove_item(&mut self, item: &String) {
        if self.ignore_items.contains(item) {
            let index = &self.ignore_items.iter().position(|x| x == item);
            let num = index.unwrap();
            self.ignore_items.remove(num);
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
pub fn load_config() -> anyhow::Result<Config> {
    info!("loading config");
    let mut config: Config = Config::default();

    //If the file doesn't exist, re-create and load defaults
    if !Path::new(CONFIG_PATH).exists() {
        warn!("No configuration found, reloading with defaults!");
        save_config(&config)?;
        return Ok(config);
    }

    // config = match toml::from_str(CONFIG_PATH) {
    //     core::result::Result::Ok(config) => config,
    //     Err(e) => {
    //Load the configuration file from stored json
    let config_file =
        fs::File::open(CONFIG_PATH).map_err(|e| anyhow!("Failed to read config file: {}", e))?;

    //Parse the lines and correct any issues
    parse_lines(&mut config, config_file);
    //Save the config
    save_config(&config)?;
    //         config
    //     },
    // };

    Ok(config)
}

///Parse each line to repair any erroneous and import any changed
fn parse_lines(config: &mut Config, file: fs::File) {
    //Read in the file
    let reader = BufReader::new(file);
    //toml splits vec to multiple lines if multiple values
    //Handle by tracking start and end values [ and ]
    let mut ignore_dir: Vec<String> = Vec::new();
    let mut read_dir = false;

    for line in reader.lines() {
        let line = match line {
            std::result::Result::Ok(line) => line,
            _ => continue,
        };
        let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();

        //Parsing the vector
        if read_dir {
            if line.contains(']') {
                //End of the vec
                read_dir = false;
                ignore_dir.extend(
                    line.trim_matches(|c| c == '[' || c == ']' || c == ' ' || c == '"')
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string()),
                );
            } else if line.contains(" = ") {
                //If the line is erroneous
                read_dir = false;
            } else {
                ignore_dir.extend(
                    line.split(',')
                        .map(|s| s.trim().trim_matches('"').to_string()),
                );
            }
            //If the vec is erroneous and doesn't end
            if !line.contains(" = ") {
                continue;
            }
        }

        //Parse each key/value pair if they exist and import them
        if let [key, value] = parts.as_slice() {
            match *key {
                "database_path" => {
                    config.database_path = value
                        .trim_matches(|c| c == '"' || c == '\'' || c == '\\')
                        .trim()
                        .to_string();
                }
                "ignore_directories" => {
                    if value.starts_with('[') && value.ends_with(']') {
                        let value: Vec<String> = value
                            .trim_matches(|c| c == '[' || c == ']' || c == ' ' || c == '"')
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .collect();
                        ignore_dir = value.to_owned();
                    } else if value.starts_with('[') || value.starts_with(']') {
                        read_dir = true;
                        let value: Vec<String> = value
                            .trim_matches(|c| c == '[' || c == ']' || c == ' ' || c == '"')
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .collect();
                        ignore_dir = value.to_owned();
                    }
                }
                "retain" => config.retain = value.parse().unwrap_or(config.retain),
                "zstd_level" => config.zstd_level = value.parse().unwrap_or(config.zstd_level),
                _ => (),
            }
        }
    }
    //Add the ignore directory vec
    if !ignore_dir.is_empty() {
        config.ignore_items = ignore_dir.into_iter().filter(|s| !s.is_empty()).collect();
    }
}

///Saves the configuration file
pub fn save_config(config: &Config) -> anyhow::Result<()> {
    info!("saving config");
    //Serialize config
    let serialized_config = toml::to_string_pretty(&config)?;
    fs::write(CONFIG_PATH, serialized_config)?;
    Ok(())
}
