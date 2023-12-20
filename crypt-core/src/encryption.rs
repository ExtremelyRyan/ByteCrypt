use anyhow::Result;
use blake2::{Blake2s256, Digest};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use log::*;

use super::filecrypt::FileCrypt;

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

/// Computes a 256-bit BLAKE2s hash for the given byte slice contents.
///
/// # Arguments
///
/// * `contents` - A reference to a slice of bytes representing the data to be hashed.
///
/// # Returns
///
/// Returns a fixed-size array of 32 bytes representing the computed hash.
///
/// # Example
///
/// ```no_run
/// let data = b"Hello, World!";
/// let hash_result = compute_hash(data);
/// println!("Computed Hash: {:?}", hash_result);
/// ```
/// # Panics
///
/// The function may panic if there are issues with the BLAKE2s hashing algorithm.
pub fn compute_hash(contents: &[u8]) -> [u8; 32] {
    info!("computing hash");
    // compute hash on contents
    let mut hasher = Blake2s256::new();
    hasher.update(contents);
    hasher.finalize().into()
}

/// compress is the Zstd compression algorithm (https://en.wikipedia.org/wiki/Zstd) to deflate file size
/// prior to encryption.
///
/// # Level
/// `level` range is from -7 (fastest, least compressed) to 22 (time intensive, most compression). Default
/// `level` is 3.
///
/// # Example
/// ```
/// # use crypt_lib::util::common::get_file_bytes;
/// # use crypt_lib::util::encryption::compress;
/// let contents: Vec<u8> = get_file_bytes("dracula.txt");
/// let compressed_contents: Vec<u8> = compress(contents.as_slice(), 3);
/// assert_ne!(contents.len(), compressed_contents.len());
/// ```
pub fn compress(contents: &[u8], level: i32) -> Vec<u8> {
    zstd::encode_all(contents, level).expect("failed to zip contents")
}

/// decompression of a file during decryption
///
/// # Example
/// ```
/// # use crypt_lib::util::common::get_file_bytes;
/// # use crypt_lib::util::encryption::{compress,decompress};
/// let contents: Vec<u8> = get_file_bytes("dracula.txt");
/// let compressed_contents: Vec<u8> = compress(contents.as_slice(), 3);
/// assert_ne!(contents.len(), compressed_contents.len());
/// let decompressed: Vec<u8> = decompress(compressed_contents.as_slice());
/// assert_eq!(contents, decompressed);
/// ```
pub fn decompress(contents: &[u8]) -> Vec<u8> {
    zstd::decode_all(contents).expect("failed to unzip!")
}

/// Seed me daddy
pub fn generate_seeds() -> ([u8; KEY_SIZE], [u8; NONCE_SIZE]) {
    let key: [u8; KEY_SIZE] = ChaCha20Poly1305::generate_key(&mut OsRng).into();
    let nonce: [u8; NONCE_SIZE] = ChaCha20Poly1305::generate_nonce(&mut OsRng).into();
    (key, nonce)
}

pub fn decrypt(fc: FileCrypt, contents: &Vec<u8>) -> Result<Vec<u8>> {
    info!("decrypting contents");
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);
    let cipher = ChaCha20Poly1305::new(k)
        .decrypt(n, contents.as_ref())
        .expect("failed to decrypt cipher text");
    Ok(cipher)
}

/// Takes a `FileCrypt` struct and encrypts the provided contents using the ChaCha20-Poly1305 cipher.
///
/// # Arguments
///
/// * `fc` - A reference to a `FileCrypt` struct containing encryption parameters, including the key and nonce.
/// * `contents` - A slice of bytes representing the contents to be encrypted.
///
/// # Returns
///
/// Returns a `Result` containing the encrypted contents as a `Vec<u8>` if successful, or an `Error` if encryption fails.
///
/// # Example
///
/// ```rust no_run
/// # use crypt_lib::{FileCrypt, encrypt};
///
/// let fc = FileCrypt::new(/* initialize FileCrypt parameters */);
/// let contents = b"Hello, World!";
/// let encrypted_contents = encrypt(&fc, contents).expect("Encryption failed!");
/// ```
/// # Panics
///
/// The function panics if encryption using ChaCha20-Poly1305 fails.
pub fn encrypt(fc: &FileCrypt, contents: &[u8]) -> Result<Vec<u8>> {
    info!("encrypting file contents");
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);
    let cipher = ChaCha20Poly1305::new(k)
        .encrypt(n, contents)
        .expect("failed to encrypt contents");
    Ok(cipher)
}
