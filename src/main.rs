mod cloud_storage;
mod database;
mod ui;
mod util;
use anyhow::{self, Ok, Result};
use ui::cli;
use util::*;
use std::path::PathBuf;

use blake2::{Blake2s256, Digest};
use hex_literal::hex;

use crate::util::path::get_full_file_path;

fn main() -> Result<()> {
    //Load config file
    let config = config::load_config().unwrap();

    //Load the UI
    let operation = cli::load_cli(config);
 
    Ok(())
}
