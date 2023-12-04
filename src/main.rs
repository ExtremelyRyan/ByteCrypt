mod cloud_storage;
mod filespawn;
mod database;
mod ui;
pub mod util;
use anyhow::{self, Result};
use ui::cli;
use util::*;
//use filespawn::*;


fn main() -> Result<()> {
    //Load config file
    let config = config::load_config().unwrap();

    //Load the UI
    //cli::load_cli(config)

    let test = filespawn::filespawn::generate_files();
    //println!("{:#?}", test);
    test
}
