mod cloud_storage;
mod database;
mod filespawn;
mod ui;
pub mod util;
use anyhow::{self, Ok, Result};
use filespawn::*;
use ui::cli;
use util::*;

fn main() -> Result<()> {
    //Load config file
    let config = config::load_config().unwrap();

    //Load the UI
    // cli::load_cli(config)
    let key = "GOOGLE_CLIENT_ID";
    match std::env::var(key) {
        core::result::Result::Ok(val) => println!("{key}: {val:?}"),
        Err(e) => std::env::set_var(key,
            "1006603075663-bi4o75nk6opljg7bicdiuden76s3v18f.apps.googleusercontent.com"),
    }
   
    // let _ = cloud_storage::oauth::google_access();

    Ok(())
}
