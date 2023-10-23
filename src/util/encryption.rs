use anyhow::{anyhow, Result};

use std::fs;

use chacha20poly1305::{aead::Aead, KeyInit, XChaCha20Poly1305};

pub fn encrypt_file(
    filepath: &str,
    dist: &str,
    key: &[u8; 32],
    nonce: &[u8; 24],
) -> Result<(), anyhow::Error> {
    let cipher = XChaCha20Poly1305::new(key.into());

    let file_data = fs::read(filepath)?;

    let encrypted_file = cipher
        .encrypt(nonce.into(), file_data.as_ref())
        .map_err(|err| anyhow!("Encrypting file: {}", err))?;

    std::fs::write(&dist, encrypted_file)?;

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
