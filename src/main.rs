use std::{fs::File, path::Path, io::{Write, Read}};
use toml;
use anyhow::{self, Result}; 
mod util;

fn main() -> Result<()> {
    // let dir = "../test_folder"; 

    // let paths = walk_directory(dir).unwrap();

    // for p in paths {
    //     let s = util::common::read_to_vec_string(p.as_str());
    //     println!("{:?} from file: {}", s, p);
    // }

    // let f = json_example().unwrap();
    let t = toml_example().unwrap();
 
    // println!("{:?}", f);
    println!("{:?}", t);
    prepend_file(t, "db.txt");

    Ok(())
}

use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct FileCrypt {
    filename: String,
    full_path: String,
    key: Vec<u8>,
    nonce: Vec<u8>, 
} 

fn toml_example() -> Result<FileCrypt> {
 
    Ok(toml::from_str(r#"
        filename = "foo.txt"
        full_path = "C:/Users/ryanm/code/bytecloak/foo.txt"
 
        key = [1,2,3,4,5]
        nonce = [6,99,7,6,6,87,5,4,6,6]

"#).unwrap() ) 
}

fn _write_to_file<P: AsRef<Path>>(path: P, file_crypt: FileCrypt) -> Result<()> {
    let mut f = File::create(path)?;
    let buf = toml::to_string(&file_crypt).unwrap();
    let bytes = buf.as_bytes();
    f.write_all(&bytes[..])?;
    Ok(())
}

/// simple prepending file
pub fn prepend_file<P: AsRef<Path> + ?Sized>(file_crypt: FileCrypt, path: &P) -> Result<()> {

    // open file
    let mut f = File::open(path)?;
    // insert new data into vec
    let mut content = toml::to_string(&file_crypt).unwrap().as_bytes().to_owned();  
    content.push(b'\n');
    f.read_to_end(&mut content)?;

    let mut f = File::create(path)?;
    f.write_all(content.as_slice())?;

    Ok(())
}