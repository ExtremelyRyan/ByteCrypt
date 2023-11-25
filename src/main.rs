mod cloud_storage;
mod ui;
mod util;
mod database;
//use std::rc::Rc;
use anyhow::{self, Ok, Result};
use util::*;
//use ui::*;         //UNCOMMENT FOR TESTING 
use database::*;


use crate::util::encryption::FileCrypt;

fn main() -> Result<()> {
    //Load config file
    config::load_config();

    //Load the UI 
    //let _ = ui::cli::load_cli();
    //let _ = tui::load_tui();  //Uncomment for TUI
    //let _ = gui::load_gui();

    let file = "foo.txt";
    let index = file.find('.').unwrap();
    let (filename, extension) = file.split_at(index);

    let fp = crate::util::path::get_full_file_path(file)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let contents: Vec<u8> = std::fs::read(file).unwrap();

    let mut fc = FileCrypt::new(filename.to_owned(), extension.to_owned(), fp);

    // generate random values for key, nonce
    fc.generate();
        
    println!("== main.rs:\n  Encrypting {} ", file);
    let mut encrypted_contents = util::encryption::encryption(&mut fc, &contents).unwrap();
    assert_ne!(contents, encrypted_contents);

    // prepend uuid to contents
    encrypted_contents = parse::prepend_uuid(&fc.uuid, &mut encrypted_contents);

    println!(
        "== main.rs\n  uuid: {}\n  as bytes: {:?}\n  len: {}",
        fc.uuid,
        fc.uuid.as_bytes(),
        fc.uuid.len()
    );

    println!("== main.rs:\n  printing first 39 characters of encrypted_contents:");
    print!("    ");
    for i in 0..39 {
        print!("{}", encrypted_contents.get(i).unwrap())
    }
    print!("\n");
    //for testing purposes, write to file
    println!("== main.rs:\n  writing encrypted file to file");
    let _ = parse::write_contents_to_file("foo.crypt", encrypted_contents);

    //write fc to crypt_keeper
    let _ = crypt_keeper::insert(&fc);

    println!("== main.rs\n  Reading data from the database");
    let crypt_collection = crypt_keeper::query(fc.uuid.clone())?;
    println!("  FileCrypt:");
    for i in 0..crypt_collection.len() {
        println!("  uuid: {:#?}\n    filename: {:#?}{:#?}", crypt_collection[i].uuid, crypt_collection[i].filename, crypt_collection[i].ext);
    }

    println!("== main.rs\n  reading contents from file");
    print!("    ");
    let file_content = std::fs::read("foo.crypt").unwrap();
    for i in 0..39 {
        print!("{:?}",file_content.get(i).unwrap());
    }
    
    
    Ok(())
}
