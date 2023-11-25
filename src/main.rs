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
    let mut config = config::load_config().unwrap();
    println!("{:#?}", config);

    config.add_cloud_service("google".to_string());

    println!("Add google {:#?}", config.cloud_services);

    config.remove_cloud_service("google".to_string());

    println!("Remove google {:#?}", config.cloud_services);

    //Resets the database (TEMPORARY) -- keeps filling up with new files
    let _ = crypt_keeper::delete_keeper()?;

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

    //for testing purposes, write to file
    println!("== main.rs:\n  writing encrypted file to file");
    let _ = parse::write_contents_to_file("foo.crypt", encrypted_contents);

    //write fc to crypt_keeper
    let _ = crypt_keeper::insert_crypt(&fc);

    println!("== main.rs  Query crypt by uuid ==");
    let crypt = crypt_keeper::query_crypt(fc.uuid.clone())?;
    //let crypt_collection = crypt_keeper::query_keeper()?;
    println!("  FileCrypt:");
    println!("    uuid: {:#?}\n    filename: {:#?}{:#?}", crypt.uuid, crypt.filename, crypt.ext);
    println!("    nonce_seed: {:?}", crypt.nonce);

    println!("== main.rs\n  reading contents from file");
    print!("    ");
    let file_content = std::fs::read("foo.crypt").unwrap();
    for i in 0..39 {
        print!("{:?}",file_content.get(i).unwrap());
    }
    
    

    //Test file #2 ---Creates a lot of spam, delete/comment out if you want
    let file2 = "bar.txt";
    let index2 = file2.find('.').unwrap();
    let (filename2, extension2) = file2.split_at(index2);

    let fp2 = crate::util::path::get_full_file_path(file2)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let contents2: Vec<u8> = std::fs::read(file2).unwrap();

    let mut fc2 = FileCrypt::new(filename2.to_owned(), extension2.to_owned(), fp2);

    // generate random values for key, nonce
    fc2.generate();
        
    println!("\n== main.rs:\n  Encrypting {} ", file2);
    let mut encrypted_contents2 = util::encryption::encryption(&mut fc2, &contents2).unwrap();
    assert_ne!(contents2, encrypted_contents2);

    // prepend uuid to contents
    encrypted_contents2 = parse::prepend_uuid(&fc2.uuid, &mut encrypted_contents2);

    //for testing purposes, write to file
    println!("== main.rs:\n  writing encrypted file to file");
    let _ = parse::write_contents_to_file("bar.crypt", encrypted_contents2);

    //write fc to crypt_keeper
    let _ = crypt_keeper::insert_crypt(&fc2);

    println!("== main.rs  Reading data from the database ==");
    let crypt2 = crypt_keeper::query_keeper()?;
    println!("  FileCrypt:");
    for i in 0..crypt2.len() {
        println!("    File {:?}:", i);
        println!("\tuuid: {:#?}\n\tfilename: {:#?}{:#?}\n\tfull_path: {:#?}\n\tkey_seed: {:?}\n\tnonce_seed: {:?}",
            crypt2[i].uuid,
            crypt2[i].filename, crypt2[i].ext,
            crypt2[i].full_path,
            crypt2[i].key,
            crypt2[i].nonce);
    }

    let _ = crypt_keeper::delete_crypt(fc2.uuid.clone())?;

    println!("== main.rs  Reading data from the database ==");
    let crypt2 = crypt_keeper::query_keeper()?;
    println!("  FileCrypt:");
    for i in 0..crypt2.len() {
        println!("    File {:?}:", i);
        println!("\tuuid: {:#?}\n\tfilename: {:#?}{:#?}\n\tfull_path: {:#?}\n\tkey_seed: {:?}\n\tnonce_seed: {:?}",
            crypt2[i].uuid,
            crypt2[i].filename, crypt2[i].ext,
            crypt2[i].full_path,
            crypt2[i].key,
            crypt2[i].nonce);
    }

    //------------------------------ Comment out to here


    Ok(())
}
