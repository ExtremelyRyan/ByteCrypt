use std::{fs, path::Path};
// use toml::Table;
use serde::{Deserialize, Serialize};

const CONFIG_PATH: &str = "src/util/config.toml";

#[derive(Deserialize, Serialize, Debug)]
///Holds the configuration for the program
pub struct Config {
    pub database_path: String,
    pub cloud_services: Vec<String>,
    pub foo: u16,
    pub bar: bool,
    pub baz: String,
    pub boom: Option<u64>,
}

///Default configuration
impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: "src/database/crypt_keeper.ds".to_string(),
            cloud_services: Vec::new(),
            foo: 5,
            bar: true,
            baz: "hello".to_string(),
            boom: Some(1),
        }
    }
}

impl Config { //Should I be returning anyhow error handling things?
    fn new(database_path: String, cloud_services: Vec<String>, 
        foo: u16, bar: bool, baz: String, boom: Option<u64>) -> Self {
         Self {
            database_path,
            cloud_services,
            foo,
            bar,
            baz,
            boom,
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

    return Ok(config);
}

///Saves the configuration file
pub fn save_config(config: &Config) -> anyhow::Result<()> {
    //Serialize config
    let serialized_config = toml::to_string_pretty(&config)?;
    fs::write(CONFIG_PATH, serialized_config)?;

    return Ok(());
}

