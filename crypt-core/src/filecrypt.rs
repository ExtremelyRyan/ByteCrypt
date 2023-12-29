use crate::{
    common::get_full_file_path,
    common::{get_crypt_folder, get_file_bytes, write_contents_to_file},
    config::get_config,
    db::{insert_crypt, query_crypt},
    encryption::decrypt,
};
use anyhow::Result;
use log::*;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::encryption::{
    compress, compute_hash, decompress, encrypt, generate_seeds, KEY_SIZE, NONCE_SIZE,
};

pub enum EncryptErrors {
    HashFail(String),
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FileCrypt {
    pub uuid: String,
    pub filename: String,
    pub ext: String,
    pub drive_id: String,
    pub full_path: PathBuf,
    pub key: [u8; KEY_SIZE],
    pub nonce: [u8; NONCE_SIZE],
    pub hash: [u8; KEY_SIZE],
}

impl FileCrypt {
    pub fn new(
        filename: String,
        ext: String,
        drive_id: String,
        full_path: PathBuf,
        hash: [u8; 32],
    ) -> Self {
        // generate key & nonce
        let (key, nonce) = generate_seeds();

        // generate file uuid
        let uuid = generate_uuid();

        Self {
            filename,
            full_path,
            drive_id,
            key,
            nonce,
            ext,
            uuid,
            hash,
        }
    }

    pub fn set_drive_id(&mut self, drive_id: String) {
        self.drive_id = drive_id;
    }
}

pub fn decrypt_file(path: &str, output: Option<String>) -> Result<(), EncryptErrors> {
    let conf = get_config();
    // get path to encrypted file
    let fp = get_full_file_path(path);
    let parent_dir = &fp.parent().unwrap().to_owned();

    // rip out uuid from contents
    let content = std::fs::read(path).expect("failed to read decryption file!");
    let (uuid, contents) = get_uuid(&content);

    // query db with uuid
    let fc = query_crypt(uuid).unwrap();
    let fc_hash: [u8; 32] = fc.hash.to_owned();

    // get output file
    let file = generate_output_file(&fc, output, parent_dir);

    let mut decrypted_content = decrypt(fc.clone(), &contents.to_vec()).expect("failed decryption");

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

pub fn decrypt_contents(fc: FileCrypt, contents: Vec<u8>) -> Result<(), EncryptErrors> {

    let fc_hash: [u8; 32] = fc.hash.to_owned();

    // get output file
    let file = generate_output_file(&fc, None, &Path::new("."));

    let (_uuid, stripped_contents) = get_uuid(&contents);

    let mut decrypted_content = decrypt(fc.clone(), &stripped_contents.to_vec()).expect("failed decryption");

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
    Ok(())
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
/// ```ignore
/// # use crypt_lib::{Config, load_config};
/// # use crypt_lib::encryption::{encrypt_file};
///
/// let path = "/path/to/your/file.txt";
/// encrypt_file(&conf, path, false);
/// ```
pub fn encrypt_file(path: &str, in_place: bool) {
    let conf = get_config();
    // parse out file path
    let (fp, parent_dir, filename, extension) = get_file_info(path);

    // get contents of file
    let binding = get_file_bytes(path);
    let mut contents = binding.as_slice();

    let fc = FileCrypt::new(
        filename,
        extension,
        "".to_string(),
        fp,
        compute_hash(contents),
    );

    // zip contents
    let binding = compress(contents, conf.zstd_level);
    contents = binding.as_slice();

    let mut encrypted_contents = encrypt(&fc, contents).unwrap();

    // prepend uuid to contents
    encrypted_contents = prepend_uuid(&fc.uuid, &mut encrypted_contents);

    let crypt_file = match in_place || !conf.retain {
        true => format!("{}/{}{}", parent_dir.display(), fc.filename, fc.ext),
        false => format!("{}/{}.crypt", &parent_dir.display(), fc.filename),
    };
    // if we are backing up crypt files, then do so.
    if conf.backup {
        let mut path = get_crypt_folder();
        // make sure we append the filename, dummy.
        path.push(format!("{}{}", fc.filename, ".crypt"));

        write_contents_to_file(path.to_str().unwrap(), encrypted_contents.clone())
            .expect("failed to write contents to backup!");
    }

    // write to file
    write_contents_to_file(&crypt_file, encrypted_contents)
        .expect("failed to write contents to file!");

    // write fc to crypt_keeper
    insert_crypt(&fc).expect("failed to insert FileCrypt data into database!");
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
/// ```ignore
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

/// Generates a Universally Unique Identifier (UUID) incorporating a timestamp and random bytes.
///
/// # Returns
///
/// Returns a string representation of the generated UUID.
///
/// # Example
///
/// ```ignore
/// # use crypt_lib::encryption::generate_uuid;
///
/// let uuid_string = generate_uuid();
/// println!("Generated UUID: {}", uuid_string);
/// ```
/// # Panics
///
/// The function may panic if the system time cannot be retrieved or if the random bytes generation fails.
pub fn generate_uuid() -> String {
    info!("generating new uuid");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();

    let mut random_bytes = [0u8; 10];
    chacha20poly1305::aead::OsRng.fill_bytes(&mut random_bytes);

    uuid::Builder::from_unix_timestamp_millis(ts.as_millis().try_into().unwrap(), &random_bytes)
        .into_uuid()
        .to_string()
}

/// gets UUID from encrypted file contents.
pub fn get_uuid(contents: &[u8]) -> (String, Vec<u8>) {
    let (uuid, contents) = contents.split_at(36);
    (
        String::from_utf8(uuid.to_vec()).unwrap_or(String::from_utf8_lossy(uuid).to_string()),
        contents.to_vec(),
    )
}

/// Prepends a UUID represented as a string to a vector of encrypted contents. Modifies vector in place.
///
/// # Arguments
///
/// * `uuid` - A string slice representing the UUID to prepend.
/// * `encrypted_contents` - A mutable reference to a vector of bytes containing encrypted contents.
///
/// # Returns
///
/// Returns a new vector of bytes with the UUID prepended to the original encrypted contents.
///
/// # Examples
///
/// ```
/// use crypt_core::filecrypt::prepend_uuid;
///
/// let mut encrypted_data = vec![1, 2, 3];
/// let uuid = "550e8400-e29b-41d4-a716-446655440000";
///
/// let result = prepend_uuid(uuid, &mut encrypted_data);
///
/// assert_eq!(result.len(), encrypted_data.len() + 36); // UUID is 36 bytes
/// assert_eq!(&result[0..36], uuid.as_bytes());        // Check if UUID is prepended correctly
/// assert_eq!(&result[36..], encrypted_data.as_slice()); // Check if original contents are preserved
/// ```
pub fn prepend_uuid(uuid: &str, encrypted_contents: &mut Vec<u8>) -> Vec<u8> {
    let mut uuid_bytes = uuid.as_bytes().to_vec();
    let mut encc = encrypted_contents.clone();
    uuid_bytes.append(&mut encc);
    uuid_bytes
}

/// given a path, dissect and return it's full path, parent folder path, filename, and extension.
///
/// # Example
/// <b>assuming current working directory is `C:/test/folder1/`</b>
/// ```ignore
/// # use crypt_lib::encryption::get_file_info;
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
    let fp = get_full_file_path(path);
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
    use crate::config::load_config;
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_encrypt_decrypt_file() {
        let mut config = load_config().unwrap();
        config.retain = true;
        encrypt_file("../dracula.txt", false);
        assert!(Path::new("../dracula.crypt").exists());
        thread::sleep(Duration::from_secs(1));
        _ = decrypt_file("../dracula.crypt", None);
        match config.retain {
            true => {
                assert!(Path::new("../dracula-decrypted.txt").exists());
                _ = std::fs::remove_file("../dracula.crypt");
                _ = std::fs::remove_file("../dracula-decrypted.txt");
            }
            false => assert!(Path::new("../dracula.txt").exists()),
        }
    }

    #[test]
    fn test_get_uuid() {
        let contents: Vec<u8> = vec![
            1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4,
            5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5,
        ];
        let res_uuid: String = String::from_utf8(vec![
            1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4,
            5, 1, 2, 3, 4, 5, 1,
        ])
        .unwrap();
        assert_eq!(get_uuid(&contents).0, res_uuid);
    }
}
