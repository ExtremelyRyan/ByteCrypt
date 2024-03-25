use crate::{
    common::{self, get_machine_name, send_information},
    db::{self},
    prelude::*,
};
use chrono::prelude::*;
use lazy_static::lazy_static;
use logfather::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::RwLock};

lazy_static! {
    ///Config path pointing to default home
    pub static ref CONFIG_PATH: String = {
        let mut path = common::get_config_folder();
        path.push(".config");


        if !path.exists() {
            _ = std::fs::create_dir(&path);
        }
        path.push("config.toml");
        format!("{}", path.display())
    };

    ///Config path pointing to default home
    pub static ref CRYPT_PATH: String = {
        let path = common::get_crypt_folder();
        if !path.exists() {
            _ = std::fs::create_dir(&path);
        }
        format!("{}", path.display())
    };

    ///Loads and holds config for session
    static ref CONFIG: RwLock<Config> = RwLock::new({
        match load_config() {
            Ok(config) => config,
            Err(err) => panic!("Failed to load config: {}", err),
        }
    });

    static ref INTERFACE: RwLock<Interface> = RwLock::new(Interface::None);

    pub static ref LOG_PATH: String = {
        let mut path = common::get_config_folder();
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

pub fn init(interface: Interface) {
    set_interface(&interface);
    load_logger(&interface);
    _ = get_config();
    _ = db::get_keeper();
}

fn load_logger(interface: &Interface) {
    let mut logger = Logger::new().file(true).path(LOG_PATH.as_str());
    logger.log_format("[{timestamp} {level}] {module_path}: {message}");
    // logger.log_format("[{timestamp} {level}] <cyan>{module_path}</cyan>: {message}");
    logger.timezone(logfather::TimeZone::Utc);
    logger.terminal(false);

    match interface {
        Interface::CLI => (),
        _ => {
            logger.terminal(false);
        }
    }
}

#[derive(Clone)]
pub enum Interface {
    None,
    CLI,
    TUI,
    GUI,
}

pub fn get_interface() -> Interface {
    INTERFACE
        .read()
        .expect("Cannot read interface type")
        .clone()
}

pub fn set_interface(interface_type: &Interface) {
    let mut interface = INTERFACE.write().expect("Cannot write interface type");
    *interface = interface_type.clone();
}

pub fn get_config() -> Config {
    CONFIG.read().expect("Cannot read config, locked").clone()
}

pub fn get_config_write() -> std::sync::RwLockWriteGuard<'static, Config> {
    CONFIG.write().expect("Cannot write to config, locked")
}

#[derive(Deserialize, Serialize, Debug, Clone)]
///Holds the configuration for the program
pub struct Config {
    /// serves as the default location for the SQLite database path.
    pub database_path: String,

    /// serves as the default location for the .crypt files path.
    pub crypt_path: String,

    ///boolean to ignore hidden files (begining with .)
    pub ignore_hidden: bool,

    ///items added to ignore_hidden will be ignored from encryption and cloud services.
    pub ignore_items: Vec<String>,

    /// PC name for current computer
    pub hwid: String,

    /// option to retain both the original file after encryption,
    /// as well as the .crypt file after decryption.
    /// if true, retains original file and encrypted file.
    /// if false, deletes files after encryption / decryption.
    // pub retain: bool,

    // /// option to retain a backup copy of all `*.crypt` files into a backup folder for
    // /// redundant storage. This only keeps the LATEST version, to not take up too much
    // /// space.
    // pub backup: bool,

    /// zstd level is for file compression, from [fastest, least compression]
    /// to [slowest, highest compression] `-7 to 22`. Default compression level is 3.
    pub zstd_level: i32,
}

///Enum for storing each item in the config struct
///
/// # Options:
/// * `ConfigTask::DatabasePath`
/// * `ConfigTask::IgnoreItems`
/// * `ConfigTask::Retain`
/// * `ConfigTask::Backup`
/// * `ConfigTask::ZstdLevel`
///
pub enum ConfigOptions {
    DatabasePath,
    CryptPath,
    IgnoreHidden,
    IgnoreItems,
    Hwid,
    ZstdLevel,
}

impl ToString for ConfigOptions {
    fn to_string(&self) -> String {
        match self {
            Self::DatabasePath => "database_path".to_string(),
            Self::IgnoreHidden => "ignore_hidden".to_string(),
            Self::IgnoreItems => "ignore_items".to_string(),
            Self::Hwid => "hwid".to_string(),
            Self::ZstdLevel => "zstd_level".to_string(),
            Self::CryptPath => "crypt_path".to_string(),
        }
    }
}

///Tasks for changing configuration
///
/// # Options:
///```ignore
/// # use crypt_lib::util::directive::ConfigTask;
/// ConfigTask::DatabasePath
/// ConfigTask::CryptPath
/// ConfigTask::IgnoreItems(ItemTask, String)
/// ConfigTask::ZstdLevel(i32)
/// ConfigTask::LoadDefault
///```
pub enum ConfigTask {
    DatabasePath,
    CryptPath,
    IgnoreHidden(bool),
    IgnoreItems(ItemsTask, String),
    Hwid,
    ZstdLevel(i32),
    LoadDefault,
}

///Ignore_items standard options
///
/// # Options
///```ignore
/// # use crypt_lib::util::directive::ItemsTask;
/// ItemsTask::Add
/// ItemsTask::Remove
///```
pub enum ItemsTask {
    Add,
    Remove,
    Default,
}

///Standard format for both CLI and TUI display
//TODO: Update for TUI purposes when completed
impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        _ = writeln!(f, "Config:");
        // _ = writeln!(f, "cloud_services: {:?}", self.cloud_services);
        _ = writeln!(f, "  database_path: {}", self.database_path);
        _ = writeln!(f, "  crypt_path: {}", self.crypt_path);
        _ = writeln!(f, "  ignore_hidden: {}", self.ignore_hidden);
        _ = writeln!(f, "  ignore_item: {:?}", self.ignore_items);
        _ = writeln!(f, "  hwid: {:?}", self.hwid);
        _ = writeln!(f, "  zstd_level: {}", self.zstd_level);
        std::fmt::Result::Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut database_path = common::get_config_folder();
        database_path.push(".config/crypt_keeper.db");
        let crypt_path = common::get_crypt_folder();
        let hwid = get_machine_name();

        Config {
            database_path: format!("{}", database_path.display()),
            crypt_path: format!("{}", crypt_path.display()),
            ignore_hidden: true,
            ignore_items: vec!["target".to_string()],
            hwid,
            zstd_level: 3,
        }
    }
}

impl Config {
    fn _new(
        database_path: String,
        crypt_path: String,
        ignore_hidden: bool,
        ignore_items: Vec<String>,
        hwid: String,
        zstd_level: i32,
    ) -> Self {
        Self {
            database_path,
            crypt_path,
            ignore_hidden,
            ignore_items,
            hwid,
            zstd_level,
        }
    }

    pub fn restore_default(&mut self) -> bool {
        *self = Config::default();

        if save_config(self).is_err() {
            return false;
        }
        true
    }

    ///Changes the database path
    pub fn change_db_path(&mut self, path: String) {
        self.database_path = path;
    }

    pub fn get_database_path(&self) -> &str {
        self.database_path.as_ref()
    }
    pub fn set_database_path(&mut self, path: &str) {
        self.database_path = path.to_owned();
        _ = save_config(self);
    }

    pub fn get_crypt_path(&self) -> &str {
        self.crypt_path.as_ref()
    }
    pub fn set_crypt_path(&mut self, path: &str) {
        self.crypt_path = path.to_owned();
        _ = save_config(self);
    }

    pub fn get_system_name(&self) -> &str {
        self.hwid.as_str()
    }
    pub fn set_system_name(&mut self, name: &str) {
        self.hwid = name.to_owned();
        _ = save_config(self);
    }

    pub fn set_ignore_hidden(&mut self, choice: bool) {
        self.ignore_hidden = choice;
    }

    pub fn get_ignore_items(&self) -> &[String] {
        self.ignore_items.as_ref()
    }

    pub fn set_ignore_items(&mut self, ignore_directories: Vec<String>) {
        self.ignore_items = ignore_directories;
        _ = save_config(self);
    }
    pub fn append_ignore_items(&mut self, item: &str) {
        self.ignore_items.push(item.to_owned());
        _ = save_config(self);
    }

    pub fn remove_ignore_item(&mut self, item: &str) {
        if self.ignore_items.contains(&item.to_string()) {
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
pub fn load_config() -> Result<Config> {
    info!("loading config");
    let mut config: Config = Config::default();

    //If the file doesn't exist, re-create and load defaults
    if !Path::new(CONFIG_PATH.as_str()).exists() {
        warning!("No configuration found, reloading with defaults!");
        save_config(&config)?;
        return Ok(config);
    }

    //Attempt to import config
    //TODO: handle more gracefully - ask user for desired change
    let content = fs::read_to_string(CONFIG_PATH.as_str())?;
    config = match toml::from_str(content.as_str()) {
        core::result::Result::Ok(config) => config,
        Err(e) => {
            send_information(vec![format!(
                "Error loading config: {}\nloading from default",
                e
            )]);

            //Save the config
            save_config(&config)?;
            config
        }
    };

    Ok(config)
}

///Saves the configuration file
pub fn save_config(config: &Config) -> Result<()> {
    info!("saving config");
    //Serialize config
    let serialized_config = toml::to_string_pretty(&config)?;
    fs::write(CONFIG_PATH.as_str(), serialized_config)?;
    Ok(())
}
