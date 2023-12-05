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
        Err(e) => std::env::set_var(key,  "1006603075663-n0o17i63am2nmcu6n77spbflrfap7l83.apps.googleusercontent.com"),
    }

    
    let key = "GOOGLE_CLIENT_SECRET";
    match std::env::var(key) {
        core::result::Result::Ok(val) => println!("{key}: {val:?}"),
        Err(e) => std::env::set_var(key,  "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"),
    }

    cloud_storage::drive::oauth_example();
    Ok(())
}
