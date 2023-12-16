use crate::{
    database::crypt_keeper,
    util::{self, common::write_contents_to_file, config::Config, *},
};
use anyhow::Result;
use blake2::Blake2s256;
use blake2::Digest;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce, Key
};
use log::*;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

pub enum EncryptErrors {
    HashFail(String),
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
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
        let key: [u8; KEY_SIZE] = ChaCha20Poly1305::generate_key(&mut OsRng).into();
        let nonce: [u8; NONCE_SIZE] = ChaCha20Poly1305::generate_nonce(&mut OsRng).into();
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
}

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

pub fn decrypt(fc: FileCrypt, contents: &Vec<u8>) -> Result<Vec<u8>> {
    info!("decrypting contents");
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);
    let cipher = ChaCha20Poly1305::new(k)
        .decrypt(n, contents.as_ref())
        .expect("failed to decrypt cipher text");
    Ok(cipher)
}

pub fn decrypt_file(
    conf: &Config,
    path: &str,
    output: Option<String>,
) -> Result<(), EncryptErrors> {
    // get path to encrypted file
    let fp = util::path::get_full_file_path(path).unwrap();
    let parent_dir = &fp.parent().unwrap().to_owned();

    // rip out uuid from contents
    let contents = std::fs::read(path).expect("failed to read decryption file!");
    let (uuid, content) = contents.split_at(36);
    let uuid_str = String::from_utf8(uuid.to_vec()).unwrap();

    // query db with uuid
    let fc = crypt_keeper::query_crypt(uuid_str).unwrap();
    let fc_hash: [u8; 32] = fc.hash.to_owned();

    // get output file
    let file = generate_output_file(&fc, output, parent_dir);

    let mut decrypted_content =
        encryption::decrypt(fc.clone(), &content.to_vec()).expect("failed decryption");

    // unzip contents
    decrypted_content = decompress(&decrypted_content);

    // compute hash on contents
    let hash = compute_hash(&decrypted_content);

    // verify file integrity
    if hash != fc_hash {
        let s = format!(
            "HASH COMPARISON FAILED\nfile hash: {:?}\ndecrypted hash:{:?}",
            &fc.hash.to_vec(),
            hash
        );
        return Err(EncryptErrors::HashFail(s));
    }

    if write_contents_to_file(&file, decrypted_content).is_err() {
        eprintln!("failed to write contents to {file}");
        std::process::exit(2);
    }

    if !conf.retain {
        std::fs::remove_file(path).unwrap_or_else(|_| panic!("failed to delete {}", path));
    }
    Ok(())
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
    info!("encrypting contents");
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);
    let cipher = ChaCha20Poly1305::new(k)
        .encrypt(n, contents)
        .expect("failed to encrypt contents");
    Ok(cipher)
}

/// Encrypts the contents of a file and performs additional operations based on the provided configuration.
///
/// # Arguments
///
/// * `conf` - A reference to a Config struct containing encryption and configuration settings.
/// * `path` - A string representing the path to the file to be encrypted.
/// * `in_place` - A boolean indicating whether to perform in-place encryption.
///
/// # Example
///
/// ```no_run
/// # use crypt_lib::util::config::{Config, load_config};
/// # use crypt_lib::util::encryption::{encrypt_file};
///
/// let conf = Config::(); // Initialize your Config struct accordingly
/// let path = "/path/to/your/file.txt";
/// encrypt_file(&conf, path, false);
/// ```
pub fn encrypt_file(conf: &Config, path: &str, in_place: bool) {
    // parse out file path
    let (fp, parent_dir, filename, extension) = get_file_info(path);

    // get contents of file
    let binding = util::common::get_file_bytes(path);
    let mut contents = binding.as_slice();

    let fc = FileCrypt::new(filename, extension, fp, compute_hash(contents));

    // zip contents
    let binding = compress(contents, conf.zstd_level);
    contents = binding.as_slice();

    let mut encrypted_contents = encrypt(&fc, contents).unwrap();

    // prepend uuid to contents
    encrypted_contents = common::prepend_uuid(&fc.uuid, &mut encrypted_contents);

    let crypt_file = match in_place || conf.retain {
        true => format!("{}/{}{}", parent_dir.display(), fc.filename, fc.ext),
        false => format!("{}/{}.crypt", &parent_dir.display(), fc.filename),
    };

    // if we are backing up crypt files, then do so.
    if conf.backup {
        let mut path = util::common::get_backup_folder();
        // make sure we append the filename, dummy.
        path.push(format!("{}{}", fc.filename, ".crypt"));

        common::write_contents_to_file(path.to_str().unwrap(), encrypted_contents.clone())
            .expect("failed to write contents to backup!");
    }

    // write to file
    common::write_contents_to_file(&crypt_file, encrypted_contents)
        .expect("failed to write contents to file!");

    //write fc to crypt_keeper
    crypt_keeper::insert_crypt(&fc).expect("failed to insert FileCrypt data into database!");

    if !conf.retain {
        std::fs::remove_file(path).unwrap_or_else(|_| panic!("failed to delete {}", path));
    }
}

/// Generates a Universally Unique Identifier (UUID) incorporating a timestamp and random bytes.
///
/// # Returns
///
/// Returns a string representation of the generated UUID.
///
/// # Example
///
/// ```rust
/// # use crypt_lib::util::encryption::generate_uuid;
///
/// let uuid_string = generate_uuid();
/// println!("Generated UUID: {}", uuid_string);
/// ```
///
/// # Panics
///
/// The function may panic if the system time cannot be retrieved or if the random bytes generation fails.
pub fn generate_uuid() -> String {
    info!("generating uuid");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();

    let mut random_bytes = [0u8; 10];
    chacha20poly1305::aead::OsRng.fill_bytes(&mut random_bytes);

    uuid::Builder::from_unix_timestamp_millis(ts.as_millis().try_into().unwrap(), &random_bytes)
        .into_uuid()
        .to_string()
}

/// Generates the output file path for decrypted content based on the provided parameters.
///
/// # Arguments
///
/// * `fc` - A reference to a `FileCrypt` struct containing file information.
/// * `output` - An optional string specifying an alternative output path or filename.
/// * `parent_dir` - A reference to the parent directory where the output file will be created.
///
/// # Returns
///
/// Returns a string representing the final output file path.
///
/// # Example
///
/// ```no_run
///
/// let fc = FileCrypt::new(/* initialize FileCrypt parameters */);
/// let parent_dir = "/path/to/parent/directory";
/// let output_file = generate_output_file(&fc, Some("/path/to/custom/output.txt".to_string()), &Path::new(parent_dir));
/// println!("Output File: {}", output_file);
/// ```
///
/// # Panics
///
/// The function may panic if there are issues with creating directories or manipulating file paths.
fn generate_output_file(fc: &FileCrypt, output: Option<String>, parent_dir: &Path) -> String {
    // default output case
    let mut file = format!("{}/{}{}", &parent_dir.display(), &fc.filename, &fc.ext);

    if Path::new(&file).exists() {
        // for now, we are going to just append the
        // filename with -decrypted to delineate between the two.
        file = format!(
            "{}/{}-decrypted{}",
            &parent_dir.display(),
            &fc.filename,
            &fc.ext
        );
    }

    // if user passes in a alternative path and or filename for us to use, use it.
    let mut p = String::new();
    if output.is_some() {
        p = output.unwrap();
    }
    if !p.is_empty() {
        let rel_path = PathBuf::from(&p);

        match rel_path.extension().is_some() {
            // 'tis a file
            true => {
                _ = std::fs::create_dir_all(rel_path.parent().unwrap());
                // get filename and ext from string
                let name = rel_path.file_name().unwrap().to_string_lossy().to_string(); // Convert to owned String
                let index = name.find('.').unwrap();
                let (filename, extension) = name.split_at(index);
                file = format!(
                    "{}/{}{}",
                    rel_path.parent().unwrap().to_string_lossy(),
                    filename,
                    extension
                );
            }
            // 'tis a new directory
            false => {
                _ = std::fs::create_dir_all(&rel_path);

                // check to make sure the last char isnt a / or \
                let last = p.chars().last().unwrap();
                if !last.is_ascii_alphabetic() {
                    p.remove(p.len() - 1);
                }
                let fp: PathBuf = PathBuf::from(p);

                file = format!("{}/{}{}", &fp.display(), &fc.filename, &fc.ext);
            }
        };
    }
    file
}

/// given a path, dissect and return it's full path, parent folder path, filename, and extension.
///
/// # Example
/// <b>assuming current working directory is `C:/test/folder1/`</b>
/// ```no_run
/// # use crypt_lib::util::encryption::get_file_info;
/// # use std::path::PathBuf;
/// let p = "file.txt";
/// let (full_path, parent, filename, extension) = get_file_info(p);
/// assert_eq!(full_path, PathBuf::from("C:\\test\\folder1\\file.txt"));
/// assert_eq!(parent,    PathBuf::from("C:\\test\\folder1"));
/// assert_eq!(filename,  "file");
/// assert_eq!(extension, ".txt");
/// ```
pub fn get_file_info(path: &str) -> (PathBuf, PathBuf, String, String) {
    // get filename, extension, and full path info
    let fp = util::path::get_full_file_path(path).unwrap();
    let parent_dir = fp.parent().unwrap().to_owned();
    let name = fp.file_name().unwrap().to_string_lossy().to_string(); // Convert to owned String
    let index = name.find('.').unwrap();
    let (filename, extension) = name.split_at(index);

    // Convert slices to owned Strings
    let filename = filename.to_string();
    let extension = extension.to_string();

    (fp, parent_dir, filename, extension)
}

// cargo nextest run
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    // #[ignore = "not working when also tested with no_retain."]
    fn test_retain_encrypt_decrypt_file() {
        let mut config = config::load_config().unwrap();
        config.retain = true;
        encrypt_file(&config, "dracula.txt", false);
        assert_eq!(Path::new("dracula.crypt").exists(), true);
        _ = decrypt_file(&config, "dracula.crypt", None);
        match config.retain {
            true => {
                assert_eq!(Path::new("dracula-decrypted.txt").exists(), true);
                _ = std::fs::remove_file("dracula.crypt");
                _ = std::fs::remove_file("dracula-decrypted.txt");
            }
            false => assert_eq!(Path::new("dracula.txt").exists(), true),
        }
    }
    #[test]
    fn test_no_retain_encrypt_decrypt_file() {
        let mut config = config::load_config().unwrap();
        config.retain = false;
        encrypt_file(&config, "dracula.txt", false);
        assert_eq!(Path::new("dracula.txt").exists(), false);
        _ = decrypt_file(&config, "dracula.crypt", None);
        match config.retain {
            true => {
                assert_eq!(Path::new("dracula-decrypted.txt").exists(), true);
                _ = std::fs::remove_file("dracula.crypt");
                _ = std::fs::remove_file("dracula-decrypted.txt");
            }
            false => assert_eq!(Path::new("dracula.txt").exists(), true),
        }
    }
}
