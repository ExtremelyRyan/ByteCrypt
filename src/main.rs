mod cloud_storage;
mod database;
mod filespawn;
mod ui;
mod util;
use anyhow::{self, Ok, Result};
use filespawn::*;
use env_logger::Builder;

use log::LevelFilter;
use ui::cli;
use util::*;

fn main() -> Result<()> {
    // change LevelFilter from trace to set the level of output messages
    Builder::new().filter_level(LevelFilter::Trace).init();

    //Load config file or get default
    let config = config::load_config().or_else(|_x| Ok(config::Config::default()))?;

    //Load the UI
    // cli::load_cli(config);
    let key = "GOOGLE_CLIENT_ID";
    match std::env::var(key) {
        core::result::Result::Ok(val) => println!("{key}: {val:?}"),
        Err(e) => std::env::set_var(key,
            "1006603075663-bi4o75nk6opljg7bicdiuden76s3v18f.apps.googleusercontent.com"),
    }
   
    let _ = cloud_storage::oauth::google_access();

    Ok(())
}
