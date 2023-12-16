mod cloud_storage;
mod database;
mod filespawn;
mod ui;
mod util;
use anyhow::{Ok, Result};
use env_logger::Builder;
use log::LevelFilter;
use ui::cli::*;
use util::*;
// use cloud_storage::*;
// use directive::*;

fn main() -> Result<()> {
    // change LevelFilter from trace to set the level of output messages
    Builder::new().filter_level(LevelFilter::Trace).init();

    //Load config file or get default
    // let config = config::load_config();

    load_cli();

    // Testing:
    // let _ = cloud_storage::oauth::google_access();

    Ok(())
}
