use anyhow::{Ok, Result};
use std::path::PathBuf;
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

///
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

pub fn get_crypt_folder() -> PathBuf {
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

///Called to print any information passed
pub fn print_information(info: Vec<String>) {
    for item in info {
        println!("{}", item);
    }
}

pub fn send_information(info: Vec<String>) {
    //TODO: Check which platform
    //CLI
    print_information(info);
    //TODO: TUI
    //TODO: GUI
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
}
