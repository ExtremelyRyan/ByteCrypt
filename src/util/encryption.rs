use crate::{
    database::crypt_keeper,
    util::{self, config::Config, parse::write_contents_to_file, *},
};
use anyhow::Result;
use blake2::{Blake2s256, Digest};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use hyper::header::ACCESS_CONTROL_ALLOW_CREDENTIALS;
use log::*;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

pub enum EncryptErrors {
    HashFail(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    /// generate key & nonce if, somehow, it was not generated using `FileCrypt::new()`
    pub fn generate(&mut self) {
        let mut k = [0u8; KEY_SIZE];
        let mut n = [0u8; NONCE_SIZE];

        OsRng.fill_bytes(&mut k);
        OsRng.fill_bytes(&mut n);

        self.key = k;
        self.nonce = n;
    }
}

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

/// takes a FileCrypt and encrypts content in place (TODO: for now)
pub fn encrypt(fc: &mut FileCrypt, contents: &[u8]) -> Result<Vec<u8>> {
    info!("encrypting contents");
    if fc.key.into_iter().all(|b| b == 0) {
        fc.generate();
    }
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);
    let cipher = ChaCha20Poly1305::new(k).encrypt(n, contents).unwrap();
    Ok(cipher)
}

pub fn encrypt_file(conf: &Config, path: &str, in_place: bool) {
    let (fp, parent_dir, filename, extension) = get_file_info(path);

    // get contents of file
    let binding = util::common::get_file_bytes(path);
    let mut contents = binding.as_slice();

    let hash = compute_hash(contents);
    // let hash = [0u8; 32]; // for benching w/o hashing only

    let mut fc = FileCrypt::new(filename, extension, fp, hash);

    // zip contents
    let binding = compress(contents, conf.zstd_level);
    contents = binding.as_slice();

    let mut encrypted_contents = encrypt(&mut fc, contents).unwrap();

    // prepend uuid to contents
    encrypted_contents = parse::prepend_uuid(&fc.uuid, &mut encrypted_contents);

    let crypt_file = match in_place {
        true => format!("{}/{}{}", parent_dir.display(), fc.filename, fc.ext),
        false => format!("{}/{}.crypt", &parent_dir.display(), fc.filename),
    };

    parse::write_contents_to_file(&crypt_file, encrypted_contents)
        .expect("failed to write contents to file!");

    //write fc to crypt_keeper
    crypt_keeper::insert_crypt(&fc).expect("failed to insert FileCrypt data into database!");

    if !conf.retain {
        std::fs::remove_file(path).unwrap_or_else(|_| panic!("failed to delete {}", path));
    }
}

/// generates a UUID v7 string using a unix timestamp and random bytes.
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
    use std::time::Duration;

    #[test]
    #[ignore = "not working when also tested with no_retain."]
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
