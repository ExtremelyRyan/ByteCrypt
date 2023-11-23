mod cloud_storage;
mod ui;
mod util;

use std::rc::Rc;

use anyhow::{self, Ok, Result};
use util::*;

use crate::util::encryption::FileCrypt;

fn main() -> Result<()> {
    //Load config file
    config::load_config();

    //Load the UI - CLI only currently
    // let _ = ui::cli::load_cli();

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
        "uuid as bytes: {:?}, len: {}",
        fc.uuid.as_bytes(),
        fc.uuid.len()
    );

    for i in 0..39 {
        print!("{}", encrypted_contents.get(i).unwrap())
    }
    //for testing purposes, write to file
    let _ = parse::write_contents_to_file("foo.crypt", encrypted_contents);

    //write fc to crypt_keeper
    let _ = parse::write_to_crypt_keeper(fc);

    let file_content = std::fs::read("foo.crypt").unwrap();
    let sub = &file_content[0..39].to_vec().to_owned();

    println!("\n\nfrom file: {:?}", String::from_utf8(sub.to_owned()));
    
    
    Ok(())
}
