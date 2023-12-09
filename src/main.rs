mod cloud_storage;
mod database;
mod filespawn;
mod ui;
mod util;

use anyhow::{self, Ok, Result};
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
    _ = cli::load_cli(config);

    Ok(())
}
