use anyhow::{anyhow, Result};
use chacha20poly1305::{aead::Aead, KeyInit, XChaCha20Poly1305};
use serde::{Deserialize, Serialize};
use std::fs::{self};

#[derive(Debug, Deserialize, Serialize)]
pub struct FileCrypt {
    filename: String,
    full_path: String,
    key: Vec<u8>,
    nonce: Vec<u8>,
}

impl FileCrypt {
    pub fn new(filename: String, full_path: String, key: Vec<u8>, nonce: Vec<u8>) -> Self {
        Self {
            filename,
            full_path,
            key,
            nonce,
        }
    }
}

pub fn encrypt_file(f: FileCrypt) -> Result<(), anyhow::Error> {
    let cipher = XChaCha20Poly1305::new(f.key.as_slice().into());
    let file_data: Vec<u8> = fs::read(f.filename)?;
    let encrypted_file: Vec<u8> = cipher
        .encrypt(f.nonce.as_slice().into(), file_data.as_ref())
        .map_err(|err| anyhow!("Encrypting file: {}", err))?;

    std::fs::write(f.full_path, encrypted_file)?;
    Ok(())
}

pub fn decrypt_file(f: FileCrypt) -> Result<(), anyhow::Error> {
    let cipher = XChaCha20Poly1305::new(f.key.as_slice().into());

    let file_data = std::fs::read(&f.full_path)?;

    let decrypted_file = cipher
        .decrypt(f.nonce.as_slice().into(), file_data.as_ref())
        .map_err(|err| anyhow!("Decrypting file: {}", err))?;

    std::fs::write(f.full_path, decrypted_file)?;

    Ok(())
}

// cargo nextest run
#[cfg(test)]
mod test {
    use super::*;
    use crate::util::common;
    use chacha20poly1305::aead::OsRng;
    use rand::RngCore;

    #[test]
    fn test_encrypt() {
        let file = "foo.txt";
        let file_crypt = "file.crypt";

        let mut key = [0u8; 32].to_vec();
        let mut nonce = [0u8; 24].to_vec();

        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);

        let fc = FileCrypt::new("foo.txt".to_string(), "file.crypt".to_string(), key, nonce);

        println!("Encrypting {} to {}", file, file_crypt);
        encrypt_file(fc).expect("encrypt failure");

        let src = common::read_to_vec_u8(file);
        let res = common::read_to_vec_u8(file_crypt);

        assert_ne!(src, res)
    }

    #[test]
    fn test_decrypt() {
        let file = "foo.txt";
        let file_crypt = "file.crypt";
        let file_decrypt = "file.decrypt";

        let mut key = [0u8; 32].to_vec();
        let mut nonce = [0u8; 24].to_vec();

        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);

        let fc = FileCrypt::new("foo.txt".to_string(), "file.crypt".to_string(), key, nonce);

        println!("Decrypting {} to {}", file_crypt, file_decrypt);
        decrypt_file(fc).expect("decrypt failure");

        let src = common::read_to_vec_u8(file);
        let res = common::read_to_vec_u8(file_decrypt);

        assert_eq!(src, res)
    }
}
