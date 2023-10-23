use anyhow::{self, Result};

use rand::{rngs::OsRng, RngCore};
use std::fs::{self};
use walkdir::WalkDir;
mod util;

fn main() -> Result<()> {
    let file = "foo.txt";
    let file_crypt = "file.crypt";
    let file_decrypt = "file.decrypt";
    println!("Removing encrypt files...");
    match fs::remove_file("file.crypt") {
        Ok(_) => {
            fs::remove_file("file.decrypt").unwrap();
        }
        Err(_) => (), // do nothing and move on.
    }

    let mut key = [0u8; 32];
    let mut nonce = [0u8; 24];
    // println!("key: {:?}\n nonce: {:?}", key, nonce);

    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    println!("Encrypting {} to {}", file, file_crypt);
    util::encryption::encrypt_file(file, file_crypt, &key, &nonce)?;

    println!("Decrypting {} to {}", file_crypt, file_decrypt);
    util::encryption::decrypt_file(file_crypt, file_decrypt, &key, &nonce)?;
 
    let cur_dir = std::env::current_dir().unwrap();
    let walker = WalkDir::new(cur_dir).into_iter();
    for entry in walker.filter_entry(|e| !util::path::is_hidden(e)) {
        let entry = entry.unwrap();
        println!("{}", entry.path().display());
    }

    Ok(())
}

// cargo nextest run
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encrypt() {
        let file = "foo.txt";
        let file_crypt = "file.crypt";
        let file_decrypt = "file.decrypt";

        let mut key = [0u8; 32];
        let mut nonce = [0u8; 24];

        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);

        println!("Encrypting {} to {}", file, file_crypt);
        util::encryption::encrypt_file(file, file_crypt, &key, &nonce).expect("encrypt failure");

        println!("Decrypting {} to {}", file_crypt, file_decrypt);
        util::encryption::decrypt_file(file_crypt, file_decrypt, &key, &nonce)
            .expect("decrypt failure");

        let src = util::common::read_to_vec_u8(file);
        let res = util::common::read_to_vec_u8(file_decrypt);

        assert_eq!(src, res)
    }
}
