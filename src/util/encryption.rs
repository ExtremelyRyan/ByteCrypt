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

pub fn decryption(fc: FileCrypt, contents: &Vec<u8>) -> Result<Vec<u8>> {
    let k = Key::from_slice(&fc.key);
    let n = Nonce::from_slice(&fc.nonce);
    let cipher = ChaCha20Poly1305::new(k)
        .decrypt(n, contents.as_ref())
        .expect("failed to decrypt cipher text");
    Ok(cipher)
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

pub enum EncryptErrors {
    HashFail(String),
}

pub fn encrypt_file(conf: &Config, path: &str, in_place: bool) {
    // get filename, extension, and full path info
    let fp = util::path::get_full_file_path(path).unwrap();
    let parent_dir = &fp.parent().unwrap().to_owned();
    let name = fp.file_name().unwrap();
    let index = name.to_str().unwrap().find('.').unwrap();
    let (filename, extension) = name.to_str().unwrap().split_at(index);

    // get contents of file
    let contents: Vec<u8> = std::fs::read(&fp).unwrap();

    // compute hash on contents
    let mut hasher = Blake2s256::new();
    hasher.update(&contents);
    let res = hasher.finalize();
    let hash: [u8; 32] = res.into();
    // let hash = [0u8; 32]; // for benching w/o hashing only

    let mut fc = FileCrypt::new(filename.to_string(), extension.to_string(), fp, hash);

    let mut encrypted_contents = encryption(&mut fc, &contents).unwrap();

    // prepend uuid to contents
    encrypted_contents = parse::prepend_uuid(&fc.uuid, &mut encrypted_contents);

    let mut crypt_file = format!("{}/{}.crypt", &parent_dir.display(), fc.filename);
    // dbg!(&crypt_file);

    if in_place {
        crypt_file = format!("{}/{}{}", parent_dir.display(), fc.filename, fc.ext);
    }
    parse::write_contents_to_file(&crypt_file, encrypted_contents)
        .expect("failed to write contents to file!");

    //write fc to crypt_keeper
    crypt_keeper::insert_crypt(&fc).expect("failed to insert FileCrypt data into database!");

    if !conf.retain {
        std::fs::remove_file(path).unwrap_or_else(|_| panic!("failed to delete {}", path));
    }
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

    let mut file = format!("{}/{}{}", &parent_dir.display(), &fc.filename, &fc.ext);
    
    if let Some(p) = output{
        if p.contains('.') {
            // we are renaming the file
            let fp = PathBuf::from(p);
            let parent = fp.parent().unwrap();
            if !parent.exists() {
                _ = std::fs::create_dir_all(s.clone());
            }
            let name = fp.file_name().unwrap();
            let index = name.to_str().unwrap().find('.').unwrap();
            let (filename, extension) = name.to_str().unwrap().split_at(index);
            file = format!("{}/{}{}", &parent.display(), &filename, &extension); 
        }else {
            // we are saving it to a new directory
            let fp = PathBuf::from(p);
            file = format!("{}/{}{}", &fp.display(), &fc.filename, &fc.ext); 
        }
    }
    dbg!(&file);

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

    let decrypted_content =
        encryption::decryption(fc.clone(), &content.to_vec()).expect("failed decryption");

    // compute hash on contents
    let mut hasher = Blake2s256::new();
    hasher.update(&decrypted_content);
    let res = hasher.finalize();

    // verify file integrity
    if res != fc_hash.into() {
        let s = format!(
            "HASH COMPARISON FAILED\nfile hash: {:?}\ndecrypted hash:{:?}",
            &fc.hash.to_vec(),
            res
        );
        return Err(EncryptErrors::HashFail(s));
    }
    // println!("hash comparison sucessful");


    write_contents_to_file(&file, decrypted_content).expect("failed writing content to file!");

    //? delete crypt file?
    if !conf.retain {
        std::fs::remove_file(path).expect("failed deleting .crypt file");
    }
    Ok(())
}

// cargo nextest run
#[cfg(test)]
mod test {
    use std::time::Duration; 
    use super::*;  

    #[test]
    #[ignore = "not working when also tested with no_retain."]
    fn test_retain_encrypt_decrypt_file() {
        let mut config = config::load_config().unwrap();
        config.retain = true;
        encrypt_file( &config, "dracula.txt", false); 
        assert_eq!(Path::new("dracula.crypt").exists(), true);
        _ = decrypt_file(&config, "dracula.crypt", None);
        match config.retain {
            true => {
                assert_eq!(Path::new("dracula-decrypted.txt").exists(), true);
                _ = std::fs::remove_file("dracula.crypt");
                _ = std::fs::remove_file("dracula-decrypted.txt");
            },
            false =>  assert_eq!(Path::new("dracula.txt").exists(), true),
        }
    }
    #[test]
    fn test_no_retain_encrypt_decrypt_file() {
        let mut config = config::load_config().unwrap();
        config.retain = false;
        encrypt_file( &config, "dracula.txt", false); 
        assert_eq!(Path::new("dracula.txt").exists(), false);
        _ = decrypt_file(&config, "dracula.crypt", None);
        match config.retain {
            true => {
                assert_eq!(Path::new("dracula-decrypted.txt").exists(), true);
                _ = std::fs::remove_file("dracula.crypt");
                _ = std::fs::remove_file("dracula-decrypted.txt");
            },
            false =>  assert_eq!(Path::new("dracula.txt").exists(), true),
        } 
    }
}
