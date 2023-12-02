use anyhow::{Ok, Result};
use std::{fs::OpenOptions, io::Write, path::PathBuf};

use super::path::get_full_file_path;

pub fn write_contents_to_file(file: &str, contents: Vec<u8>) -> Result<()> {
    let s = PathBuf::from(file);
    if !s.exists() {
        _ = std::fs::create_dir_all(s.clone());
    }

    let fp = get_full_file_path(s.display().to_string().as_str()).unwrap();
    dbg!(&fp);

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(fp.display().to_string())?;
    f.write_all(contents.as_slice())
        .expect("failed writing to file");
    Ok(f.flush()?)
}

pub fn prepend_uuid(uuid: &String, encrypted_contents: &mut Vec<u8>) -> Vec<u8> {
    let mut uuid_bytes = uuid.as_bytes().to_vec();
    uuid_bytes.append(encrypted_contents);
    uuid_bytes
}
