mod cloud_storage;
mod ui;
mod util;
use std::io::Read;

use anyhow::{self, Ok, Result};
use ui::ui::load_ui;
use util::*;

use crate::util::encryption::*;

fn main() -> Result<()> {
    // config::load_config();

    //Load the UI - CLI only currently
    // let _ = load_ui();

    // let file = "foo.txt";
    // let index = file.find('.').unwrap();
    // let (file_name, ext) = file.split_at(index);
    // let full_path = crate::util::path::get_full_file_path(file)
    //     .unwrap()
    //     .to_str()
    //     .unwrap()
    //     .to_string();
    // let contents: Vec<u8> = std::fs::read(file).unwrap();

    // let k = [0u8; KEY_SIZE];
    // let n = [0u8; NONCE_SIZE];
    // let mut fc = FileCrypt::new(file_name.to_owned(), ext.to_owned(), full_path, k, n);

    // fc.generate();

    // println!("Encrypting {} ", file);
    // let encrypted_contents = encrypt_file(&mut fc, &contents).unwrap();
    // assert_ne!(contents, encrypted_contents);

    // println!("Decrypting {} ", file);
    // let decrypted_contents = decrypt_file(fc, &encrypted_contents).unwrap();
    // assert_eq!(decrypted_contents, contents);

    let fc = parse::read_from_crypt("").unwrap();

    println!("{:?}", fc);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_path() {
        let dir = "../test_folder";
        for p in path::walk_directory(dir).unwrap() {
            let s = util::common::read_to_vec_string(p.as_str());
            println!("{:?} from file: {}", s, p);
        }
    }
}

// fn _test_write_db() -> Result<()> {
//     let t = util::parse::toml_example()?;
//     println!("{:?}", t);
//     parse::prepend_file(t, "db")
// }
