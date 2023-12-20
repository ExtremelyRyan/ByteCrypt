use anyhow::{Ok, Result};
use std::path::PathBuf;
use std::process::Command;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
};

use crate::path::get_full_file_path;
use crate::ui_repo::{CharacterSet, Color};

/// given a path, dissect and return a struct containing the full path, is_dir, parent path, and name.
#[derive(Debug, Clone)]
pub struct PathInfo {
    pub full_path: PathBuf,
    pub is_dir: bool,
    pub parent: PathBuf,
    pub name: String,
}

impl PathInfo {
    pub fn new(path: &str) -> Self {
        let full_path = match path.is_empty() {
            true => std::env::current_dir().unwrap(),
            false => get_full_file_path(path).unwrap(),
        };

        Self {
            is_dir: full_path.is_dir(),
            parent: full_path.parent().unwrap().to_owned(),
            name: full_path.file_name().unwrap().to_string_lossy().to_string(),
            full_path,
        }
    }
}

///FileSystemEntity enum
#[derive(Debug)]
pub enum FileSystemEntity {
    File(FileInfo),
    Directory(DirInfo),
}

///Directory struct
#[derive(Debug)]
pub struct DirInfo {
    pub path: PathInfo,
    pub expanded: bool,
    pub contents: Vec<FileSystemEntity>,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub id: String,
}

///Builds a file tree with given DirInfo struct
pub fn build_tree(dir_info: &DirInfo, depth: usize, last: bool) -> Vec<String> {
    let char_set = CharacterSet::U8_SLINE_CURVE;
    let dir_color = Color::Blue;
    let mut indent = String::new();
    let hline = char_set.h_line.to_string().repeat(2);
    let not_last = format!("{}   ", char_set.v_line);
    
    let mut tree: Vec<String> = Vec::new();

    if depth != 0 {
        indent = if last {
            "    ".repeat(depth)
        } else {
            if depth > 1 {
                not_last.clone() + "    ".repeat(depth - 2).as_str() + not_last.as_str()
            } else {
                not_last + "    ".repeat(depth - 1).as_str()  
            }
        };
    } else {
        tree.push(format!("{}", /*dir_color.paint*/(&dir_info.name).to_string()));
    }


    for (index, item) in dir_info.contents.iter().enumerate() {
        let is_last = index == dir_info.contents.len() - 1;
        let prefix = if is_last {char_set.node} else {char_set.joint};

        match item {
            FileSystemEntity::File(file) => {
                tree.push(format!("{}{}{} {}", indent, prefix, hline, file.name));
            },
            FileSystemEntity::Directory(subdir) => {
                tree.push(format!(
                    "{}{}{} {}", indent, prefix, hline, 
                    /*dir_color.paint*/(&subdir.name).to_string()
                ));
                let mut subtree = build_tree(subdir, depth + 1, is_last);
                tree.append(&mut subtree);
            },
        }
    }

    return tree;
}


pub fn build_tree_again(dir_info: &DirInfo, depth: usize, is_root: bool) -> Vec<String> {
    let char_set = CharacterSet::U8_SLINE_CURVE;
    let dir_color = Color::Blue;
    let joint = format!("{}{}{} ", char_set.joint, char_set.h_line, char_set.h_line);
    let node = format!("{}{}{} ", char_set.node, char_set.h_line, char_set.h_line);
    let vline = format!("{}   ", char_set.v_line);
    
    let mut tree: Vec<String> = Vec::new();

    if is_root {
        tree.push(/*dir_color.paint*/(&dir_info.name).to_string());
    }

    for (index, entity) in dir_info.contents.iter().enumerate() {
        let is_last = index == dir_info.contents.len() - 1;
        let prefix = if is_last { &node } else { &joint };

        let mut indent = String::new();
        if depth > 0 {
            indent = vline.repeat(depth - 1) + if is_last {"    "} else { vline.as_str() };
        }

        match entity {
            FileSystemEntity::File(file) => {
                tree.push(indent.clone() + prefix + &file.name);
            },
            FileSystemEntity::Directory(subdir) => {
                tree.push(indent.clone() + prefix + /*dir_color.paint*/(&subdir.name).to_string().as_str());
                tree.extend(build_tree_again(subdir, depth + 1, false));
            },
        }
        
    }

    return tree;
}


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
