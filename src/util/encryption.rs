use anyhow::{anyhow, Result}; 
use std::fs;

use chacha20poly1305::{aead::Aead, KeyInit, XChaCha20Poly1305};

pub fn encrypt_file( file_in: &str, file_out: &str, key: &[u8; 32],  nonce: &[u8; 24] ) -> Result<(), anyhow::Error> {
    let cipher = XChaCha20Poly1305::new(key.into());

    let file_data: Vec<u8> = fs::read(file_in)?;

    let encrypted_file: Vec<u8> = cipher
        .encrypt(nonce.into(), file_data.as_ref())
        .map_err(|err| anyhow!("Encrypting file: {}", err))?;

    std::fs::write(&file_out, encrypted_file)?;

    Ok(())
}

pub fn decrypt_file(
    encrypted_file_path: &str,
    dist: &str,
    key: &[u8; 32],
    nonce: &[u8; 24],
) -> Result<(), anyhow::Error> {
    let cipher = XChaCha20Poly1305::new(key.into());

    let file_data = std::fs::read(encrypted_file_path)?;

    let decrypted_file = cipher
        .decrypt(nonce.into(), file_data.as_ref())
        .map_err(|err| anyhow!("Decrypting file: {}", err))?;

    std::fs::write(&dist, decrypted_file)?;

    Ok(())
}


// cargo nextest run
#[cfg(test)]
mod test {
    use chacha20poly1305::aead::OsRng;
    use rand::RngCore;

    use crate::util::common;

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
        encrypt_file(file, file_crypt, &key, &nonce).expect("encrypt failure");

        println!("Decrypting {} to {}", file_crypt, file_decrypt);
        decrypt_file(file_crypt, file_decrypt, &key, &nonce)
            .expect("decrypt failure");

        let src = common::read_to_vec_u8(file);
        let res = common::read_to_vec_u8(file_crypt);

        assert_ne!(src, res)
    }

    #[test]
    fn test_decrypt() { 
        let file = "foo.txt";
        let file_crypt = "file.crypt";
        let file_decrypt = "file.decrypt";

        let mut key = [0u8; 32];
        let mut nonce = [0u8; 24];

        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);
 
        println!("Decrypting {} to {}", file_crypt, file_decrypt);
        decrypt_file(file_crypt, file_decrypt, &key, &nonce)
            .expect("decrypt failure");

        let src = common::read_to_vec_u8(file);
        let res = common::read_to_vec_u8(file_decrypt);

        assert_eq!(src, res)
    }
}