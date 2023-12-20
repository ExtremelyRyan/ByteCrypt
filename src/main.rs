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

fn main() -> Result<()> {
    // change LevelFilter from trace to set the level of output messages
    Builder::new().filter_level(LevelFilter::Trace).init();

    load_cli();        

    Ok(())
}
