use crate::util::path::get_full_file_path;
use anyhow::{Ok, Result};
use std::path::{PathBuf, Path};
use std::process::Command;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},

};

/// read file, and return values within a Vector of Strings.
pub fn read_to_vec_string(path: &str) -> Vec<String> {
    let f = File::options()
        .read(true)
        .append(true)
        .create(true)
        .open(path)
        .expect("Error opening file! \n");

    let reader = BufReader::new(f);
    let mut v: Vec<String> = Vec::new();
    for line in reader.lines() {
        v.push(line.unwrap());
    }
    v
}

/// read file, and return values within a Vector of Strings.
pub fn get_file_bytes(path: &str) -> Vec<u8> {
    std::fs::read(path).expect("Can't open/read file!")
}

pub fn write_contents_to_file(file: &str, contents: Vec<u8>) -> Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(file)?;
    f.write_all(contents.as_slice())
        .expect("failed writing to file");
    Ok(f.flush()?)
}

pub fn prepend_uuid(uuid: &String, encrypted_contents: &mut Vec<u8>) -> Vec<u8> {
    let mut uuid_bytes = uuid.as_bytes().to_vec();
    uuid_bytes.append(encrypted_contents);
    uuid_bytes
}


pub fn get_backup_folder() -> PathBuf {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "echo %userprofile%"])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("echo $HOME")
            .output()
            .expect("failed to execute process")
    };

    let stdout = output.stdout;
    let mut path = PathBuf::from(String::from_utf8(stdout).expect("ERROR").trim());
    path.push("crypt");

    if !path.exists() {
        _ = std::fs::create_dir(&path);
    }

    path
}

/// our hacky workarounds for converting pathbuf to string and str
pub trait Convert {
    /// using display() to convert to a String. <b>Can lose non-unicode characters!</b>
    fn string(&self) -> String;
}

impl Convert for PathBuf {
    fn string(&self) -> String {
        self.display().to_string()
    }
}

pub enum Cloud {
    Drive,
    Dropbox,
}

/// depending on which cloud provider we are using, store the token in the user environment.
pub fn get_token(cloud: Cloud) -> Option<String> {
    let key = match cloud {
        Cloud::Drive => "CRYPT_DRIVE_TOKEN",
        Cloud::Dropbox => "CRYPT_DROPBOX_TOKEN",
    };
    match std::env::var(key) {
        std::result::Result::Ok(val) => Some(val),
        Err(e) => {
            log::error!("issue getting token!: {e}");
            None
        }
    }
}

/// depending on which cloud provider we are using, store the token in the user environment.
pub fn store_token(token: &String, cloud: Cloud) {
    let key = match cloud {
        Cloud::Drive => "CRYPT_DRIVE_TOKEN",
        Cloud::Dropbox => "CRYPT_DROPBOX_TOKEN",
    };
    std::env::set_var(key, token);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_to_vec_string() {
        let s = String::from("The Project Gutenberg eBook of Dracula");
        let dracula = "./dracula.txt";
        let res = read_to_vec_string(dracula);
        assert_eq!(s, res[0]);
    }

    #[test]
    fn test_get_set_token() {
        let test_token = "abc123".to_string();
        store_token(&test_token, Cloud::Drive);
        let retrieved_token = get_token(Cloud::Drive).unwrap();
        assert_eq!(test_token, retrieved_token);

        std::env::remove_var("CRYPT_DRIVE_TOKEN".to_string());
    }
}
