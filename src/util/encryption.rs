use anyhow::{anyhow, Ok, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs::{self, File}; 


///Directory object holds file objects
#[derive(Debug)]
pub struct DirectoryCrypt {
    ///Name of the root directory being encrypted
    directory_name: String,
    ///Vector of FileCrypts being encrypted
    files: Vec<FileCrypt>,
    //other variables
}
///Implementation for DirectoryCrypt
impl DirectoryCrypt {
    pub fn new(directory_name: String, files: Vec<FileCrypt>) -> Self {
        Self {
            directory_name,
            files,
        }
    }
}

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

#[derive(Debug, Deserialize, Serialize)]
pub struct FileCrypt {
    pub filename: String,
    pub ext: String,
    pub full_path: String,
    key: [u8; KEY_SIZE],
    nonce: [u8; NONCE_SIZE],
}

impl FileCrypt {
    pub fn new(
        filename: String,
        ext: String,
        full_path: String,
        key: [u8; KEY_SIZE],
        nonce: [u8; NONCE_SIZE],
    ) -> Self {
        Self {
            filename,
            full_path,
            key,
            nonce,
            ext,
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
        let fc: FileCrypt = match serde_json::from_str(s.as_str()) {
            Ok(f) => f,
            Err(e) => panic!("ERROR: {e}"), 
        };
        Self {
            filename: fc.filename,
            ext: fc.ext,
            full_path: fc.full_path,
            key: fc.key,
            nonce: fc.nonce,
        }
    }
}

/// takes a FileCrypt and encrypts content in place (TODO: for now)
pub fn encrypt_file(fc: &mut FileCrypt, contents: &Vec<u8>) -> Result<Vec<u8>> {
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

pub fn decrypt_file(fc: FileCrypt, contents: &Vec<u8>) -> Result<Vec<u8>> {
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);

    let cipher = ChaCha20Poly1305::new(k)
        .decrypt(n, contents.as_ref())
        .unwrap();
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
    use crate::util::common;

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

        // generate new key and nonce palceholders
        let k = [0u8; KEY_SIZE];
        let n = [0u8; NONCE_SIZE];
        let mut fc = FileCrypt::new(filename.to_owned(), extension.to_owned(), fp, k, n);

        // generate random values for key, nonce
        fc.generate();

        println!("Encrypting {} ", file);
        let encrypted_contents = encrypt_file(&mut fc, &contents).unwrap();
        assert_ne!(contents, encrypted_contents);
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

        // generate new key and nonce palceholders
        let k = [0u8; KEY_SIZE];
        let n = [0u8; NONCE_SIZE];
        let mut fc = FileCrypt::new(filename.to_owned(), extension.to_owned(), fp, k, n);

        // generate random values for key, nonce
        fc.generate();

        println!("Encrypting {} ", file);
        decrypt_file(fc, &contents).expect("decrypt failure");

        let src = common::read_to_vec_u8(file);
        let res = common::read_to_vec_u8(file_decrypt);

        assert_eq!(src, res)
    }
}
