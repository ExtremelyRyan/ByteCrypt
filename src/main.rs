mod cloud_storage;
mod database;
mod filespawn;
mod ui;
mod util;
use anyhow::{Ok, Result};
use crypt_lib::util::path::get_full_file_path;
use env_logger::Builder;
use log::LevelFilter;
use ui::cli::*;
use util::*;

fn main() -> Result<()> {
    // change LevelFilter from trace to set the level of output messages
    Builder::new().filter_level(LevelFilter::Trace).init();

    //Load config file or get default
    let config = config::load_config().or_else(|_x| Ok(config::Config::default()))?;

    // _ = filespawn::file_generator::generate_files();

    _ = load_cli(config);

    // zip
    // let contents = common::get_file_bytes("dracula.txt");
    // let start = Instant::now();
    // let compressed = util::encryption::compress(contents.clone().as_slice(), 3);
    // let duration = start.elapsed();
    // _ = util::parse::write_contents_to_file("d.zipped", compressed.clone());

    // println!("Time elapsed in zstd is: {:?} ", duration);

    // println!("contents len: {:?} ", contents.len());
    // println!("compressed len: {:?} ", compressed.len());

    //Load the UI
    // cli::load_cli(config);
    // let key = "GOOGLE_CLIENT_ID";
    // match std::env::var(key) {
    //     core::result::Result::Ok(val) => println!("{key}: {val:?}"),
    //     Err(e) => std::env::set_var(key,
    //         "1006603075663-bi4o75nk6opljg7bicdiuden76s3v18f.apps.googleusercontent.com"),
    // }

    // let _ = cloud_storage::oauth::google_access();


    // testing query DB for existing file: 
    let (full_path) = get_full_file_path("dracula.txt").expect("Can't find full path for file");
    let fc = database::crypt_keeper::query_keeper_for_existing_file(full_path);
    // thoughts:
    // is this worthwhile? I can imagine for someone who does not really move files around a lot, that this could save
    // a fair bit of extra entries from redundancy. 
    // but on the flip side, as soon as they rename the file, or move directories, we are creating a "redundant" entry anyway.
    

    println!("{:#?}", fc);


    Ok(())
}
