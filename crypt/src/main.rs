use crypt_ui::cli::load_cli;
use env_logger::Builder;
use log::LevelFilter;

fn main() -> anyhow::Result<()> {
    // change LevelFilter from trace to set the level of output messages
    Builder::new().filter_level(LevelFilter::Off).init();

    load_cli();

    Ok(())
}
