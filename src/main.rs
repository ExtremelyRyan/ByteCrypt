mod cloud_storage;
mod database;
mod filespawn;
mod ui;
pub mod util;

use anyhow::{self, Result};
use env_logger::{Builder, WriteStyle};
use filespawn::*;
use log::{debug, error, info, trace, warn, LevelFilter};
use ui::cli;
use util::*;

fn main() -> Result<()> {
    Builder::new().filter_level(LevelFilter::Trace).init();
    //Load config file
    let config = config::load_config().unwrap();

    //Load the UI
    cli::load_cli(config);

    Ok(())
}
