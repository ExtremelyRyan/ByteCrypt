mod cloud_storage;
mod ui;
mod util;
mod database;
use std::rc::Rc;
use anyhow::{self, Ok, Result};
use util::*;
use ui::*;
use database::*;


use crate::util::encryption::FileCrypt;

fn main() -> Result<()> {
    //Load config file
    config::load_config();

    //Load the UI - CLI only currently
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

    println!("Encrypting {} ", file);
    let mut encrypted_contents = util::encryption::encryption(&mut fc, &contents).unwrap();
    assert_ne!(contents, encrypted_contents);

    // prepend uuid to contents
    encrypted_contents = parse::prepend_uuid(&fc.uuid, &mut encrypted_contents);

    println!(
        "uuid: {} as bytes: {:?}, len: {}",
        fc.uuid,
        fc.uuid.as_bytes(),
        fc.uuid.len()
    );

    println!("printing first 39 characters of encrypted_contents:");
    for i in 0..39 {
        print!("{}", encrypted_contents.get(i).unwrap())
    }
    print!("\n");
    //for testing purposes, write to file
    println!("writing encrypted file to file");
    let _ = parse::write_contents_to_file("foo.crypt", encrypted_contents);

    //write fc to crypt_keeper
    let _ = parse::write_to_crypt_keeper(fc);
    println!("reading contents from file");
    let file_content = std::fs::read("foo.crypt").unwrap();
    //let sub = &file_content[0..300].to_vec().to_owned();

    for i in 0..300 {
        print!("{:?}",file_content.get(i).unwrap());
    }
    //println!("\nfrom file: {:?}", String::from_utf8(sub.to_vec()));
    
    
    Ok(())
}
