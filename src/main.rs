use chrono;
use crypt_ui::cli::load_cli;
use std::time::UNIX_EPOCH;
// use env_logger::Builder;
// use log::LevelFilter;

fn main() -> anyhow::Result<()> {
    // change LevelFilter from trace to set the level of output messages
    // Builder::new().filter_level(LevelFilter::Trace).init();

    load_cli();

    Ok(())
}
