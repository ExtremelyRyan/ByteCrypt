mod cloud_storage;
mod database;
mod filespawn;
mod ui;
pub mod util;
use anyhow::{self, Ok, Result};
use filespawn::*;
use ui::cli;
use util::*;

fn main() -> Result<()> {
    //Load config file
    let config = config::load_config().unwrap();

    //Load the UI
    // cli::load_cli(config)
    let key = "GOOGLE_CLIENT_ID";
    match std::env::var(key) {
        core::result::Result::Ok(val) => println!("{key}: {val:?}"),
        Err(e) => std::env::set_var(key, 
    }
   
    // let test = cloud_storage::drive::google_drive_access();
    let test = cloud_storage::oauth::google_access();
    println!("test: {:?}", test);

    Ok(())
}
