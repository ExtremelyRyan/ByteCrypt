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

    pub fn generate(&mut self) {
        let mut k = [0u8; KEY_SIZE];
        let mut n = [0u8; NONCE_SIZE];

        OsRng.fill_bytes(&mut k);
        OsRng.fill_bytes(&mut n);

        self.key = k;
        self.nonce = n;
    }
}

pub fn compute_hash(contents: &Vec<u8>) -> [u8; 32] {
    // compute hash on contents
    let mut hasher = Blake2s256::new();
    hasher.update(contents);
    hasher.finalize().into()
}

pub fn decryption(fc: FileCrypt, contents: &Vec<u8>) -> Result<Vec<u8>> {
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
    let contents: Vec<u8> = std::fs::read(&fp).unwrap();
    let (uuid, content) = contents.split_at(36);
    let uuid_str = String::from_utf8(uuid.to_vec()).unwrap();

    // query db with uuid
    let fc = crypt_keeper::query_crypt(uuid_str).unwrap();
    let fc_hash: [u8; 32] = fc.hash.to_owned();

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
    if output.is_some() { p = output.unwrap(); }
    if !p.is_empty() {
        let rel_path = PathBuf::from(&p);
        
        match rel_path.extension().is_some() {
            // 'tis a file
            true => {
                 _ = std::fs::create_dir_all(&rel_path.parent().unwrap());
                // get filename and ext from string
                let name = rel_path.file_name().unwrap().to_string_lossy().to_string(); // Convert to owned String
                let index = name.find('.').unwrap();
                let (filename, extension) = name.split_at(index);
                file = format!("{}/{}{}", rel_path.parent().unwrap().to_string_lossy().to_string(), filename, extension);
            },
            // 'tis a new directory
            false =>  {
                _ = std::fs::create_dir_all(&rel_path);
                
                // check to make sure the last char isnt a / or \
                let last = p.chars().last().unwrap();
                if !last.is_ascii_alphabetic() {
                    p.remove(p.len() - 1);
                }
                let fp: PathBuf = PathBuf::from(p);

                file = format!("{}/{}{}", &fp.display(), &fc.filename, &fc.ext);
            },
        };
    } 

    let decrypted_content =
        encryption::decryption(fc.clone(), &content.to_vec()).expect("failed decryption");

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

    write_contents_to_file(&file, decrypted_content).expect("failed writing content to file!");

    //? delete crypt file?
    if !conf.retain {
        std::fs::remove_file(path).expect("failed deleting .crypt file");
    }
    Ok(())
}

/// takes a FileCrypt and encrypts content in place (TODO: for now)
pub fn encryption(fc: &mut FileCrypt, contents: &Vec<u8>) -> Result<Vec<u8>> {
    if fc.key.into_iter().all(|b| b == 0) {
        fc.generate();
    }
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);
    let cipher = ChaCha20Poly1305::new(k)
        .encrypt(n, contents.as_ref())
        .unwrap();
    Ok(cipher)
}



pub fn encrypt_file(conf: &Config, path: &str, in_place: bool) {
    let (fp, parent_dir, filename, extension) = get_file_info(path);

    // get contents of file
    let contents: Vec<u8> = std::fs::read(&fp).unwrap();

    let hash = compute_hash(&contents);
    // let hash = [0u8; 32]; // for benching w/o hashing only

    let mut fc = FileCrypt::new(
        filename,
        extension,
        fp,
        hash,
    );

    let mut encrypted_contents = encryption(&mut fc, &contents).unwrap();

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




/// generates a UUID 7 string using a unix timestamp and random bytes.
pub fn generate_uuid() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();

    let mut random_bytes = [0u8; 10];
    chacha20poly1305::aead::OsRng.fill_bytes(&mut random_bytes);

    uuid::Builder::from_unix_timestamp_millis(ts.as_millis().try_into().unwrap(), &random_bytes)
        .into_uuid()
        .to_string()
}



fn get_file_info(path: &str) -> (PathBuf, PathBuf, String, String) {
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
