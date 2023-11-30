use std::path::PathBuf;

use anyhow::{Ok, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

#[derive(Debug, Deserialize, Serialize)]
pub struct FileCrypt {
    pub uuid: String,
    pub filename: String,
    pub ext: String,
    pub full_path: PathBuf,
    pub key: [u8; KEY_SIZE],
    pub nonce: [u8; NONCE_SIZE],
    pub hash: [u8; KEY_SIZE],
}

impl FileCrypt {
    pub fn new(filename: String, ext: String, full_path: PathBuf, hash: [u8; 32]) -> Self {
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
            hash,
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

/// generates a UUID 7 string using a unix timestamp and random bytes.
pub fn generate_uuid() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();

    let mut random_bytes = [0u8; 10];
    chacha20poly1305::aead::OsRng.fill_bytes(&mut random_bytes);

    uuid::Builder::from_unix_timestamp_millis(ts.as_millis().try_into().unwrap(), &random_bytes)
        .into_uuid()
        .to_string()
}

// // cargo nextest run
// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::util::{common, parse};

//     #[test]
//     fn test_encrypt() {
//         let file = "dracula.txt";
//         let index = file.find('.').unwrap();
//         let (filename, extension) = file.split_at(index);

//         let fp = crate::util::path::get_full_file_path(file).unwrap();
//         let contents: Vec<u8> = std::fs::read(file).unwrap();

//         let mut fc = FileCrypt::new(filename.to_owned(), extension.to_owned(), fp);

//         // generate random values for key, nonce
//         fc.generate();

//         println!("Encrypting {} ", file);
//         let mut encrypted_contents = encryption(&mut fc, &contents).unwrap();
//         assert_ne!(contents, encrypted_contents);

//         // prepend uuid to contents
//         encrypted_contents = parse::prepend_uuid(&fc.uuid, &mut encrypted_contents);

//         //for testing purposes, write to file
//         let _ = parse::write_contents_to_file("dracula.crypt", encrypted_contents);

//         //write fc to crypt_keeper
//     }

//     #[test]
//     fn test_decrypt() {
//         let file = "dracula.crypt";
//         let index = file.find('.').unwrap();
//         let (filename, extension) = file.split_at(index);

//         let fp = crate::util::path::get_full_file_path(file).unwrap();
//         let contents: Vec<u8> = std::fs::read(file).unwrap();

//         let fc: FileCrypt = /*crate::database::crypt_keeper::query_crypt(fc.uuid.clone())?;*/
//              FileCrypt::new(filename.to_string(), extension.to_string(), fp.clone());

//         dbg!(&fc);

//         println!("Encrypting {} ", file);
//         let decryped_contents = decryption(fc, &contents).expect("decrypt failure");

//         let src = common::read_to_vec_u8("dracula.txt");

//         assert_eq!(src, decryped_contents);
//     }
// }
