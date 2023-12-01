mod cloud_storage;
mod database;
mod ui;
mod util;
use anyhow::{self, Result};
use ui::cli;
use util::*;

fn main() -> Result<()> {
    //Load config file
    let config = config::load_config().unwrap();

    //Load the UI
    cli::load_cli(config)
}
