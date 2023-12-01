use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const CONFIG_PATH: &str = "config.toml";

#[derive(Deserialize, Serialize, Debug)]
///Holds the configuration for the program
pub struct Config {
    /// serves as the default location for the SQLite database path.
    pub database_path: String,
    /// collection of cloud services currently holding crypt files.
    pub cloud_services: Vec<String>,
    /// option to retain both the original file after encryption,
    /// as well as the .crypt file after decryption.
    /// if true, retains original file and encrypted file.
    /// if false, deletes files after encryption / decryption.
    pub retain: bool,
    // collection of any directories to ignore during folder encryption.
    pub ignore_directories: Vec<String>,
}

///Default configuration
impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: "crypt_keeper.db".to_string(),
            cloud_services: Vec::new(),
            retain: true,
            ignore_directories: Vec::new(),
        }
    }
}

impl Config {
    //Should I be returning anyhow error handling things?
    fn new(
        database_path: String,
        cloud_services: Vec<String>,
        retain: bool,
        hidden_directories: Vec<String>,
    ) -> Self {
        Self {
            database_path,
            cloud_services,
            retain,
            ignore_directories: hidden_directories,
        }
    }

    ///Changes the database path
    pub fn change_db_path(&mut self, path: String) {
        self.database_path = path;
    }

    ///Adds a cloud service to the list
    pub fn add_cloud_service(&mut self, service: String) {
        self.cloud_services.push(service);
    }

    ///Removes a cloud service from the list
    pub fn remove_cloud_service(&mut self, service: String) {
        self.cloud_services.retain(|s| s != &service);
    }

    pub fn get_database_path(&self) -> &str {
        self.database_path.as_ref()
    }
}

///Loads configuration file -- creates default if missing
pub fn load_config() -> anyhow::Result<Config> {
    //If the file doesn't exist, re-create and load defaults
    if !Path::new(CONFIG_PATH).exists() {
        println!("No configuration found, reloading with defaults!");
        let config = Config::default();
        save_config(&config)?;
    }

    //Load the configuration file from stored json
    let config_file = fs::read_to_string(CONFIG_PATH)?;
    let config: Config = toml::from_str(&config_file)?;

    Ok(config)
}

///Saves the configuration file
pub fn save_config(config: &Config) -> anyhow::Result<()> {
    //Serialize config
    let serialized_config = toml::to_string_pretty(&config)?;
    fs::write(CONFIG_PATH, serialized_config)?;

    Ok(())
}
