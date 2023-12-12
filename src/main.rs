mod cloud_storage;
mod database;
mod filespawn;
mod ui;
mod util;
use anyhow::{self, Error, Ok, Result};
use env_logger::Builder;
use filespawn::*;

use log::LevelFilter;
use ui::cli;
use util::*;
 
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::util::encryption::file_zip;

fn main() -> Result<()> {
    // change LevelFilter from trace to set the level of output messages
    Builder::new().filter_level(LevelFilter::Trace).init();

    //Load config file or get default
    let config = config::load_config().or_else(|_x| Ok(config::Config::default()))?;

    // _ = filespawn::file_generator::generate_files();

    // _ = ui::cli::load_cli(config);

    // zip
    let contents = common::get_file_bytes("dracula.txt");
    let start = Instant::now();
    let compressed = util::encryption::compress(contents.as_slice());
    let duration = start.elapsed();
    _ = util::parse::write_contents_to_file("d.zipped", compressed);
    

    println!("Time elapsed in zstd is: {:?} ", duration);

    let start = Instant::now();
    encryption::file_zip("dracula.txt");
    let duration = start.elapsed();

    println!("Time elapsed in Powershell zip is: {:?} ", duration);

    //Load the UI
    // cli::load_cli(config);
    // let key = "GOOGLE_CLIENT_ID";
    // match std::env::var(key) {
    //     core::result::Result::Ok(val) => println!("{key}: {val:?}"),
    //     Err(e) => std::env::set_var(key,
    //         "1006603075663-bi4o75nk6opljg7bicdiuden76s3v18f.apps.googleusercontent.com"),
    // }

    // let _ = cloud_storage::oauth::google_access();

    Ok(())
}




