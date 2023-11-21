use anyhow::{anyhow, Result, Ok};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce, Key
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    option,
};

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

#[derive(Debug)]
pub struct FileCrypt {
    pub filename: String,
    pub full_path: String,
    key: Option<Key>,
    nonce: Option<Nonce>,
}



impl FileCrypt {
    pub fn new(filename: String, full_path: String, key: Option<Key>, nonce: Option<Nonce>) -> Self 
    { Self { filename, full_path, key, nonce } }

 
    pub fn generate(&mut self) {
        self.key = Some(ChaCha20Poly1305::generate_key(&mut OsRng));
        self.nonce = Some(ChaCha20Poly1305::generate_nonce(&mut OsRng)); // 192-bits; unique per message
    }
}

/// takes a FileCrypt and encrypts content in place (TODO: for now)
pub fn encrypt_file(mut fc: FileCrypt) -> Result<(), anyhow::Error> {
 
    if fc.key.is_none() || fc.nonce.is_none() { f.generate(); }
    

    let cipherfilename
     = ChaCha20Poly1305::new(&f.key.unwrap());
    let file_data: Vec<u8> = fs::read(&f.filename)?;
    let encrypted_file: Vec<u8> = cipher
        .encrypt(&f.nonce.unwrap(), file_data.as_ref())
        .map_err(|err| anyhow!("Encrypting file: {}", err))?;

    std::fs::write("file.crypt", encrypted_file)?;

    // write filecrypt to file
    // crate::util::parse::prepend_file(f, "crypt")
    Ok(())
}

pub fn decrypt_file(f: FileCrypt) -> Result<(), anyhow::Error> {
    let cipher = ChaCha20Poly1305::new(f.key.unwrap().as_slice().into());

    let file_data = std::fs::read(&f.full_path)?;

    let decrypted_file = cipher
        .decrypt(f.nonce.unwrap().as_slice().into(), file_data.as_ref())
        .map_err(|err| anyhow!("Decrypting file: {}", err))?;

    std::fs::write(f.full_path, decrypted_file)?;

    Ok(())
}

// cargo nextest run
#[cfg(test)]
mod test {
    use super::*;
    use crate::util::common;

    #[test]
    fn test_encrypt() {
        let file = "foo.txt";
        let file_crypt = "file.crypt";
 

        let fc = FileCrypt::new(
            "foo.txt".to_string(),
            "".to_string(),
            None,
            None
        );

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

        let fc = FileCrypt::new(
            "foo.txt".to_string(),
            "file.crypt".to_string(),
            None,
            None
        );

        println!("Decrypting {} to {}", file_crypt, file_decrypt);
        decrypt_file(fc).expect("decrypt failure");

        let src = common::read_to_vec_u8(file);
        let res = common::read_to_vec_u8(file_decrypt);

        assert_eq!(src, res)
    }
}
