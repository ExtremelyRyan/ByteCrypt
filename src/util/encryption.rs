use anyhow::{Ok, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use super::uuid::generate_uuid;

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

#[derive(Debug, Deserialize, Serialize)]
pub struct FileCrypt {
    pub uuid: String,
    pub filename: String,
    pub ext: String,
    pub full_path: String,
    pub key: [u8; KEY_SIZE],
    pub nonce: [u8; NONCE_SIZE],
}

impl FileCrypt {
    pub fn new(filename: String, ext: String, full_path: String) -> Self {
        // generate key & nonce
        let mut key = [0u8; KEY_SIZE];
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);

        // generate file uuid
        let uuid = generate_uuid();

        Self {
            filename,
            full_path,
            key,
            nonce,
            ext,
            uuid,
        }
    }

    pub fn generate(&mut self) {
        let mut k = [0u8; KEY_SIZE];
        let mut n = [0u8; NONCE_SIZE];

        OsRng.fill_bytes(&mut k);
        OsRng.fill_bytes(&mut n);

        self.key = k;
        self.nonce = n;
    }

    pub fn from_string(s: String) -> Self {
        let fc: FileCrypt = serde_json::from_str(s.as_str()).unwrap();
        Self {
            filename: fc.filename,
            ext: fc.ext,
            full_path: fc.full_path,
            key: fc.key,
            nonce: fc.nonce,
            uuid: fc.uuid,
        }
    }
}

/// takes a FileCrypt and encrypts content in place (TODO: for now)
pub fn encryption(fc: &mut FileCrypt, contents: &Vec<u8>) -> Result<Vec<u8>> {
    if fc.key.into_iter().all(|b| b == 0) {
        fc.generate();
    }
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);
    let cipher = ChaCha20Poly1305::new(k)
        .encrypt(n, contents.as_ref())
        .unwrap();
    Ok(cipher)
}

pub fn decryption(fc: FileCrypt, contents: &Vec<u8>) -> Result<Vec<u8>> {
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);
    let cipher = ChaCha20Poly1305::new(k)
        .decrypt(n, contents.as_ref())
        .expect("failed to decrypt cipher text");
    Ok(cipher)
}

// {

//     let extension = fc.filename.find('.').unwrap();
//     let fname = fc.filename.split_at(extension);

//     std::fs::write(format!("{}.decrypt", fname.0), decrypted_file)?;

//     Ok(())
// }

// cargo nextest run
#[cfg(test)]
mod test {
    use super::*;
    use crate::util::{
        common,
        parse::{self, read_crypt_keeper},
    };

    #[test]
    fn test_encrypt() {
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
        let mut encrypted_contents = encryption(&mut fc, &contents).unwrap();
        assert_ne!(contents, encrypted_contents);

        // prepend uuid to contents
        encrypted_contents = parse::prepend_uuid(&fc.uuid, &mut encrypted_contents);

        //for testing purposes, write to file
        let _ = parse::write_contents_to_file("foo.crypt", encrypted_contents);

        //write fc to crypt_keeper
        let _ = parse::write_to_crypt_keeper(fc);
    }

    #[test]
    fn test_decrypt() {
        let file = "foo.crypt";
        let index = file.find('.').unwrap();
        let (filename, extension) = file.split_at(index);

        let fp = crate::util::path::get_full_file_path(file)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let contents: Vec<u8> = std::fs::read(file).unwrap();
        let crypts = read_crypt_keeper().unwrap();

        let mut fc: FileCrypt =
            FileCrypt::new(filename.to_string(), extension.to_string(), fp.clone());

        for c in crypts {
            if c.uuid == fp {
                fc.uuid = c.uuid; 
            }
        }

        dbg!(&fc);

        println!("Encrypting {} ", file);
        let decryped_contents = decryption(fc, &contents).expect("decrypt failure");

        let src = common::read_to_vec_u8("foo.txt");

        assert_eq!(src, decryped_contents);
    }
}
