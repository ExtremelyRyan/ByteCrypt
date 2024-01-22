use crate::{
    common::get_full_file_path,
    common::{
        chooser, get_crypt_folder, get_file_bytes, walk_crypt_folder, write_contents_to_file,
    },
    config::get_config,
    db::{insert_crypt, query_crypt},
    encryption::{
        compress, compute_hash, decompress, decrypt, encrypt, generate_seeds, KEY_SIZE, NONCE_SIZE,
    },
};
use anyhow::Result;
use logfather::*;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    fs::read,
    io,
    path::{Path, PathBuf},
    time::Duration,
};

/// Represents various errors that can occur during file decryption.
///
/// The `FcError` enum provides specific error variants for different failure scenarios
/// encountered during the decryption process.
///
/// # Variants
///
/// - `HashFail(String)`: Hash comparison between file and decrypted content failed.
/// - `InvalidFilePath`: The provided file path is invalid.
/// - `CryptQueryError`: Failed to query the cryptographic information.
/// - `DecompressionError`: Failed to decompress the decrypted content.
/// - `FileDeletionError(std::io::Error, String)`: Failed to delete the original file.
/// - `FileReadError`: An error occurred while reading the file.
/// - `FileError(String)`: An error occurred during file operations (read or write).
/// - `DecryptError(String)`: Failed to decrypt the file contents.
///
/// # Examples
///
/// ```rust ignore
/// use crypt_core::FcError;
///
/// fn handle_error(err: FcError) {
///     match err {
///         FcError::HashFail(message) => eprintln!("Hash failure: {}", message),
///         FcError::InvalidFilePath => eprintln!("Invalid file path."),
///         FcError::CryptQueryError => eprintln!("Cryptographic query failed."),
///         FcError::DecompressionError => eprintln!("Decompression failed."),
///         FcError::FileDeletionError(io_err, path) => eprintln!("Failed to delete file {}: {:?}", path, io_err),
///         FcError::FileReadError => eprintln!("Error reading file."),
///         FcError::FileError(message) => eprintln!("File operation error: {}", message),
///         FcError::DecryptError(message) => eprintln!("Decryption error: {}", message),
///     }
/// }
/// ```
///
#[derive(Debug)]
pub enum FcError {
    HashFail(String),
    InvalidFilePath,
    CryptQueryError,
    DecompressionError(String),
    FileDeletionError(std::io::Error, String),
    FileReadError,
    FileError(String),
    DecryptError(String),
    GeneralError(String),
}

impl From<String> for FcError {
    fn from(err_msg: String) -> Self {
        FcError::GeneralError(err_msg)
    }
}

/// Represents cryptographic information associated with an encrypted file.
///
/// The `FileCrypt` struct contains details such as the UUID, filename, extension, drive ID,
/// full path, encryption key, nonce, and hash of an encrypted file.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FileCrypt {
    /// The UUID associated with the encrypted file.
    pub uuid: String,

    /// The filename of the encrypted file.
    pub filename: String,

    /// The extension of the encrypted file.
    pub ext: String,

    /// The drive ID associated with the encrypted file.
    pub drive_id: String,

    /// The full path of the encrypted file.
    pub full_path: PathBuf,

    /// The encryption key used to encrypt the file.
    pub key: [u8; KEY_SIZE],

    /// The nonce used in the encryption process.
    pub nonce: [u8; NONCE_SIZE],

    /// The hash of the encrypted file.
    pub hash: [u8; KEY_SIZE],
}

impl FileCrypt {
    /// Creates a new `FileCrypt` instance with the provided parameters. Generates random `key`, `nonce`, and `UUID` during creation.
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename of the encrypted file.
    /// * `ext` - The extension of the encrypted file.
    /// * `drive_id` - The drive ID associated with the encrypted file.
    /// * `full_path` - The full path of the encrypted file.
    /// * `hash` - The hash of the encrypted file.
    ///
    /// # Returns
    /// A new `FileCrypt` instance with generated `UUID`, `key`, and `nonce`.
    pub fn new(
        filename: String,
        ext: String,
        drive_id: String,
        full_path: PathBuf,
        hash: [u8; KEY_SIZE],
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

    /// Sets the drive ID associated with the encrypted file.
    ///
    /// # Arguments
    ///
    /// * `drive_id` - The new drive ID value to set.
    pub fn set_drive_id(&mut self, drive_id: String) {
        self.drive_id = drive_id;
    }
}

/// Decrypts a file using ChaCha20Poly1305 encryption and verifies its integrity.
///
/// # Arguments
///
/// * `filename` - filename of the .crypt file residing in the crypt folder.
/// * `output` - An optional output path for the decrypted content.
/// * `conf` - An optional configuration, if not provided, the default configuration is used.
///
/// # Returns
///
/// A `Result<(), FcError>` indicating success or an error with details.
///
/// # Errors
///
/// This function returns various error types under the `FcError` enum:
/// - `InvalidFilePath`: The provided file path is invalid.
/// - `FileError(String)`: An error occurred while reading or writing the file.
/// - `CryptQueryError`: Failed to query the cryptographic information.
/// - `DecryptError(String)`: Failed to decrypt the file contents.
/// - `DecompressionError`: Failed to decompress the decrypted content.
/// - `HashFail(String)`: Hash comparison between file and decrypted content failed.
/// - `FileDeletionError(std::io::Error, String)`: Failed to delete the original file.
/// # Panics
///
/// This function may panic in case of critical errors, but most errors are returned in the `Result`.
pub fn decrypt_file(filename: &str, output: Option<String>) -> Result<(), FcError> {
    // get location of crypt folder and append "decrypted" path
    let mut crypt_folder = get_crypt_folder();

    // walk along crypt folder and find all files.
    let paths = match walk_crypt_folder() {
        Ok(p) => p,
        Err(e) => panic!("{e}"),
    };

    let mut compared: Vec<PathBuf> = Vec::new();

    // appeasing the compiler gods
    let binding = filename.trim().to_lowercase();
    let filename = binding.as_str();

    // compare files found to filename, and keep in compared those that match
    for p in paths.iter() {
        // file may or may not include extension, so check for both & if filename is partial match.
        if p.file_stem().unwrap().to_ascii_lowercase() == filename
            || p.file_name().unwrap().to_ascii_lowercase() == filename
            || p.to_string_lossy()
                .to_string()
                .to_lowercase()
                .contains(filename)
        {
            compared.push(p.to_owned());
        }
    }

    // if we have more than one match, prompt user to choose which file they want.
    let file_match = match compared.len() > 1 {
        true => chooser(compared, filename),
        false => compared[0].to_owned(),
    };

    let content = read(file_match).map_err(|e| FcError::FileError(e.to_string()))?;

    let (uuid, contents) = get_uuid(&content)?;

    let fc = match query_crypt(uuid) {
        Ok(f) => f,
        Err(e) => panic!("{}", e.to_string()),
    };

    let fc_hash: [u8; 32] = fc.hash.to_owned();

    // make sure we put decrypted file in the "decrypted" folder, dummy.
    crypt_folder.push("decrypted");
    let file = generate_output_file(&fc, output, &crypt_folder);
    dbg!(&file);

    let mut decrypted_content = match decrypt(fc.clone(), &contents.to_vec()) {
        Ok(d) => d,
        Err(e) => return Err(FcError::DecryptError(e.to_string())),
    };

    decrypted_content = match decompress(&decrypted_content) {
        Ok(d) => d,
        Err(e) => return Err(FcError::DecompressionError(e.to_string())),
    };

    let hash = compute_hash(&decrypted_content);

    if hash != fc_hash {
        let s = format!(
            "HASH COMPARISON FAILED\nfile hash: {:?}\ndecrypted hash:{:?}",
            &fc.hash.to_vec(),
            hash
        );
        return Err(FcError::HashFail(s));
    }

    write_contents_to_file(&file, decrypted_content)
        .map_err(|e| FcError::FileError(e.to_string()))?;

    Ok(())
}

pub fn decrypt_contents(fc: FileCrypt, contents: Vec<u8>) -> Result<(), FcError> {
    let fc_hash: [u8; 32] = fc.hash.to_owned();

    // get location of crypt folder and append "decrypted" path
    let mut crypt_folder = get_crypt_folder();
    crypt_folder.push("decrypted");

    // get output file
    let file = generate_output_file(&fc, None, Path::new(&crypt_folder));

    let (_uuid, stripped_contents) = get_uuid(&contents)?;

    let mut decrypted_content =
        decrypt(fc.clone(), &stripped_contents.to_vec()).expect("failed decryption");

    // unzip contents
    decrypted_content = match decompress(&decrypted_content) {
        Ok(d) => d,
        Err(_) => todo!(),
    };

    // compute hash on contents
    let hash = compute_hash(&decrypted_content);

    // verify file integrity
    if hash != fc_hash {
        let s = format!(
            "HASH COMPARISON FAILED\nfile hash: {:?}\ndecrypted hash:{:?}",
            &fc.hash.to_vec(),
            hash
        );
        return Err(FcError::HashFail(s));
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
pub fn encrypt_file(path: &str, output: &Option<String>) {
    let conf = get_config();
    // parse out file path
    let (fp, _, filename, extension) = get_file_info(path);

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

    let mut path = get_crypt_folder();
    match output {
        Some(o) => {
            let mut alt_path = path.clone();
            alt_path.push(o);
            dbg!(&alt_path, &o);
            if !PathBuf::from(&alt_path).exists() {
                match std::fs::create_dir_all(&alt_path) {
                    Ok(_) => (),
                    Err(e) => panic!("{}", e.to_string()),
                }
            }
            path.push(format!(r#"{}\{}{}"#, o, fc.filename, ".crypt"));
        }
        None => path.push(format!("{}{}", fc.filename, ".crypt")),
    }

    // write fc to crypt_keeper
    insert_crypt(&fc).expect("failed to insert FileCrypt data into database!");

    write_contents_to_file(path.to_str().unwrap(), encrypted_contents.clone())
        .expect("failed to write contents to file!");
}

pub fn encrypt_contents(path: &str) -> Option<Vec<u8>> {
    if path.contains(".crypt") {
        return None;
    }
    let conf = get_config();
    // parse out file path
    let (fp, _, filename, extension) = get_file_info(path);

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

    // write crypt file to crypt folder
    let mut path = get_crypt_folder();
    // make sure we append the filename, dummy.
    path.push(format!("{}{}", fc.filename, ".crypt"));

    // TODO: fix this later.
    match write_contents_to_file(path.to_str().unwrap(), encrypted_contents.clone()) {
        Ok(_) => (),
        Err(_) => todo!(),
    }

    // TODO: fix this later.
    // write fc to crypt_keeper
    match insert_crypt(&fc) {
        Ok(_) => (),
        Err(_) => todo!(),
    }

    Some(encrypted_contents)
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
/// # Panics
///
/// The function may panic if there are issues with creating directories or manipulating file paths.
fn generate_output_file(fc: &FileCrypt, output: Option<String>, parent_dir: &Path) -> String {
    // default output case
    let mut file = format!("{}/{}{}", &parent_dir.display(), &fc.filename, &fc.ext);

    if !Path::new(&parent_dir).exists() {
        _ = std::fs::create_dir(parent_dir);
    }

    // if we already have an existing file, we will loop and count up until we find a verison that is not there
    if Path::new(&file).exists() {
        let mut counter = 1;
        // dont know if this is the right path at the moment, but works for now.
        loop {
            file = format!(
                "{}/{}({}){}",
                &parent_dir.display(),
                &fc.filename,
                counter,
                &fc.ext
            );
            if Path::new(&file).exists() {
                counter += 1;
            } else {
                break;
            }
        }
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
                if cfg!(target_os = "windows") {
                    file = format!(
                        "{}\\{}{}",
                        rel_path.parent().unwrap().to_string_lossy(),
                        filename,
                        extension
                    );
                } else {
                    file = format!(
                        "{}/{}{}",
                        rel_path.parent().unwrap().to_string_lossy(),
                        filename,
                        extension
                    );
                }
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
                if cfg!(target_os = "windows") {
                    file = format!("{}\\{}{}", &fp.display(), &fc.filename, &fc.ext);
                } else {
                    file = format!("{}/{}{}", &fp.display(), &fc.filename, &fc.ext);
                }
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
/// ```rust
/// # use crypt_core::filecrypt::generate_uuid;
///
/// let uuid_string = generate_uuid();
/// println!("Generated UUID: {}", uuid_string);
/// ```
/// # Panics
/// The function may panic if the system time cannot be retrieved or if the random bytes generation fails.
pub fn generate_uuid() -> String {
    info!("generating new uuid");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::new(63871342634, 0));

    let mut random_bytes = [0u8; 10];
    chacha20poly1305::aead::OsRng.fill_bytes(&mut random_bytes);

    uuid::Builder::from_unix_timestamp_millis(ts.as_millis().try_into().unwrap(), &random_bytes)
        .into_uuid()
        .to_string()
}

/// Extracts a UUID and the remaining contents from a byte slice.
///
/// # Arguments
///
/// * `contents` - A slice of `u8` bytes containing the UUID and additional data.
///
/// # Returns
///
/// A `Result` containing a tuple with the extracted UUID as a `String` and the
/// remaining contents as a `Vec<u8>`. If the input is too short to extract the UUID,
/// an `Err` variant is returned with an error message.
///
/// # Examples
///
/// ```rust
///  # use crypt_core::filecrypt::get_uuid;
/// let contents = b"123e4567-e89b-12d3-a456-426614174001rest_of_data";
/// let result = get_uuid(contents);
/// assert!(result.is_ok());
/// let (uuid, rest) = result.unwrap();
/// println!("UUID: {}", uuid);
/// println!("Remaining Data: {:?}", rest);
/// assert_eq!(uuid, "123e4567-e89b-12d3-a456-426614174001");
/// ```
///
/// # Errors
///
/// Returns an `Err` variant with an error message if the input is too short to extract the UUID.
///
/// # Panics
///
/// The function will panic if the length of `contents` is less than 36.
pub fn get_uuid(contents: &[u8]) -> Result<(String, Vec<u8>), String> {
    if contents.len() < 36 {
        return Err("Input too short to extract UUID".to_string());
    }

    let (uuid, contents) = contents.split_at(36);
    Ok((
        String::from_utf8(uuid.to_vec()).unwrap_or(String::from_utf8_lossy(uuid).to_string()),
        contents.to_vec(),
    ))
}

/// Reads a file specified by the provided path and extracts a UUID from its contents.
///
/// # Arguments
///
/// * `file` - A type that implements `AsRef<Path>`, representing the path to the file.
///
/// # Returns
///
/// Returns a `Result` containing a `String` with the extracted UUID if successful,
/// or a `Box<dyn std::error::Error>` containing an error if the operation fails.
///
/// # Errors
///
/// The function may return an error in the following cases:
///
/// * The file has an invalid extension (not "crypt").
/// * The file has no extension.
/// * The file content cannot be read.
/// * The UTF-8 conversion of the file content fails.
///
/// # Example
///
/// ```rust ignore
/// # use crypt_core::filecrypt::get_uuid_from_file;
/// use std::path::Path;
/// use std::io;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let uuid = get_uuid_from_file("dracula.crypt")?;
///     println!("Extracted UUID: {}", uuid);
///     Ok(())
/// }
/// ```
pub fn get_uuid_from_file<T: AsRef<Path>>(file: T) -> Result<String, Box<dyn std::error::Error>> {
    let path = file.as_ref();

    // Check if the file has the expected extension
    match path.extension() {
        Some(ext) => match ext == "crypt" {
            true => (),
            false => {
                return Err(
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid file extension").into(),
                )
            }
        },
        None => {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "File has no extension").into())
        }
    }

    // Read the file contents
    let contents = std::fs::read(path)?;

    // Extract UUID (assuming it is the first 36 characters)
    let uuid = String::from_utf8(contents.clone().drain(0..36).collect())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UTF-8: {}", e)))?;

    Ok(uuid)
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
pub fn prepend_uuid(uuid: &str, encrypted_contents: &mut [u8]) -> Vec<u8> {
    let mut uuid_bytes = uuid.as_bytes().to_vec();
    let mut encc = encrypted_contents.to_owned();
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
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    #[ignore = "works locally, fails in CI"]
    fn test_encrypt_decrypt_file() {
        encrypt_file("crypt-core/benches/files/dracula.txt", &None);
        let mut crypt = get_crypt_folder();
        crypt.push("dracula.crypt");
        assert!(crypt.exists());

        thread::sleep(Duration::from_secs(1));

        _ = decrypt_file(crypt.to_str().unwrap(), None);

        let mut dracula_decypted = get_crypt_folder();
        dracula_decypted.push("decrypted");
        dracula_decypted.push("dracula.txt");

        assert!(dracula_decypted.exists());
        _ = std::fs::remove_file(crypt);
        _ = std::fs::remove_file(dracula_decypted);
    }

    #[test]
    fn test_get_uuid() {
        let contents: Vec<u8> = vec![
            1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4,
            5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5,
        ];
        let uuid_test: String = String::from_utf8(vec![
            1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4,
            5, 1, 2, 3, 4, 5, 1,
        ])
        .unwrap();
        let (uuid, _) = get_uuid(&contents).unwrap();
        assert_eq!(uuid, uuid_test);
    }
}
