mod cloud_storage;
mod ui;
mod util;

use anyhow::{self, Ok, Result};
use util::*;

fn main() -> Result<()> {
    //Load config file
    config::load_config();

    //Load the UI - CLI only currently
    // let _ = ui::cli::load_cli();

    Ok(())
}
