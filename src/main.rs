mod cloud_storage;
mod database;
mod ui;
mod util;
use anyhow::{self, Ok, Result};
use ui::cli;
use util::*;
use std::path::PathBuf;

use crate::util::path::get_full_file_path;

fn main() -> Result<()> {
    //Load config file
    let config = config::load_config().unwrap();

    //Load the UI
    let operation = cli::load_cli(config);

    // let path = "dracula.txt";
    // let fp = get_full_file_path(path).unwrap();
 
    // let parent = fp.parent().unwrap();

    // dbg!(parent);

    Ok(())
}
