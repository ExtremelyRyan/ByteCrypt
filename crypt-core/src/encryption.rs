use crate::{error, filecrypt::FileCrypt, prelude::*};
use blake2::{Blake2s256, Digest, *};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use logfather::*;

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
/// # Panics
///
/// The function may panic if there are issues with the BLAKE2s hashing algorithm.
pub fn compute_hash(contents: &[u8]) -> [u8; 32] {
    info!("computing hash");
    // compute hash on contents
    let mut hasher = Blake2s256::new();
    digest::Update::update(&mut hasher, contents);
    hasher.finalize().into()
}

/// compress is the Zstd compression algorithm <https://en.wikipedia.org/wiki/Zstd> to deflate file size
/// prior to encryption.
///
/// # Level
/// `level` range is from -7 (fastest, least compressed) to 22 (time intensive, most compression). Default
/// `level` is 3.
///
/// # Example
/// ```ignore
/// # use crate::common::get_file_bytes;
/// # use crate::encryption::compress;
/// let contents: Vec<u8> = get_file_bytes("dracula.txt");
/// let compressed_contents: Vec<u8> = compress(contents.as_slice(), 3);
/// assert_ne!(contents.len(), compressed_contents.len());
/// ```
pub fn compress(contents: &[u8], level: i32) -> Vec<u8> {
    zstd::encode_all(contents, level).expect("failed to zip contents")
}

/// Decompresses a byte slice using the Zstandard compression algorithm.
///
/// # Arguments
/// * `contents` - A byte slice containing the compressed data.
///
/// # Returns
/// A `Vec<u8>` containing the decompressed data.
///
/// # Panics
/// Panics if the decompression process fails.
pub fn decompress(contents: &[u8]) -> Result<Vec<u8>> {
    let contents = zstd::decode_all(contents)?;
    return Ok(contents);
}

/// Generates a random key and nonce pair for use in ChaCha20Poly1305 encryption.
///
/// # Returns
///
/// A tuple containing two arrays: the first array is the randomly generated key,
/// and the second array is the randomly generated nonce.
///
pub fn generate_seeds() -> ([u8; KEY_SIZE], [u8; NONCE_SIZE]) {
    let key: [u8; KEY_SIZE] = ChaCha20Poly1305::generate_key(&mut OsRng).into();
    let nonce: [u8; NONCE_SIZE] = ChaCha20Poly1305::generate_nonce(&mut OsRng).into();
    (key, nonce)
}

/// Decrypts a byte slice using the ChaCha20Poly1305 encryption algorithm.
///
/// # Arguments
///
/// * `fc` - A `FileCrypt` struct containing the key and nonce required for decryption.
/// * `contents` - A reference to a `Vec<u8>` containing the encrypted data.
///
/// # Returns
/// A `Result<Vec<u8>, chacha20poly1305::Error>` where the `Ok` variant contains the decrypted data on success.
///
/// # Errors
/// Returns a `chacha20poly1305::Error` if the decryption process fails.
///
/// # Panics
///
/// Panics if the decryption process encounters a critical error.
pub fn decrypt(fc: FileCrypt, contents: &Vec<u8>) -> Result<Vec<u8>> {
    info!("decrypting contents");
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);
    ChaCha20Poly1305::new(k)
        .decrypt(n, contents.as_ref())
        .map_err(|_| Error::EncryptionError(error::EncryptionError::ChaChaError))
}

/// Takes a `FileCrypt` struct and encrypts the provided contents using the ChaCha20-Poly1305 cipher.
///
/// # Arguments
///
/// * `fc` - A reference to a `FileCrypt` struct containing encryption parameters, including the key and nonce.
/// * `contents` - A slice of bytes representing the contents to be encrypted.
///
/// # Returns
/// A `Result<Vec<u8>, chacha20poly1305::Error>` where the `Ok` variant contains the decrypted data on success.
/// # Errors
/// Returns a `chacha20poly1305::Error` if the decryption process fails.
///
/// # Panics
/// The function panics if encryption using ChaCha20-Poly1305 fails.
pub fn encrypt(fc: &FileCrypt, contents: &[u8]) -> Result<Vec<u8>> {
    info!("encrypting file contents");
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);
    ChaCha20Poly1305::new(k)
        .encrypt(n, contents)
        .map_err(|_| Error::EncryptionError(error::EncryptionError::ChaChaError))
}

// cargo nextest run
#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_hash() {
        let contents = b"hello there";
        let res = compute_hash(contents);
        let hash_contents: [u8; 32] = [
            79, 124, 186, 26, 222, 68, 179, 58, 201, 141, 84, 168, 242, 8, 48, 130, 131, 223, 134,
            150, 210, 132, 93, 249, 24, 62, 200, 173, 167, 129, 67, 242,
        ];
        assert_eq!(hash_contents, res);
    }

    #[test]
    fn test_seeds() {
        let (k, n) = generate_seeds();
        assert_ne!(k, [0u8; KEY_SIZE]);
        assert_ne!(n, [0u8; NONCE_SIZE]);
    }

    #[test]
    fn test_compress_decompress() {
        let contents = b"hello there";

        let res = compress(contents, 3);

        assert_ne!(contents, res.as_slice());

        let dec = decompress(res.as_slice()).unwrap();

        assert_eq!(dec.as_slice(), contents);
    }

    #[test]
    fn test_encrypt() {
        let fc = FileCrypt::new(
            "".to_string(),
            "".to_string(),
            "".to_string(),
            PathBuf::from(""),
            [0u8; KEY_SIZE],
        );
        let contents = b"hello there";
        let res = encrypt(&fc, contents).unwrap();
        assert_ne!(contents, res.as_slice());
    }
    #[test]
    fn test_decrypt() {
        let fc = FileCrypt::new(
            "".to_string(),
            "".to_string(),
            "".to_string(),
            PathBuf::from(""),
            [0u8; KEY_SIZE],
        );
        let contents = b"hello there";
        let res = encrypt(&fc, &contents.clone()).unwrap();
        assert_ne!(contents, res.as_slice());

        let dec = decrypt(fc, &res).unwrap();
        assert_eq!(contents, dec.as_slice());
    }
}
