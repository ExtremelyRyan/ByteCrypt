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
