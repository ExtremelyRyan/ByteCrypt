use anyhow::{Ok, Result};
use std::path::PathBuf;
use std::process::Command;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
};  
use walkdir::WalkDir;

use crate::config;
use ansi_term::Color;
use crate::ui_repo::CharacterSet;

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

///Represents a file system entity
///
/// # Options:
///```no_run
/// File(FileInfo),
/// Directory(DirInfo),
///```
#[derive(Debug)]
pub enum FsNode {
    File(FileInfo),
    Directory(DirInfo),
}

///Stores information about a file
///
///```no_run
/// FileInfo {
///     name: String, //Name of the file
///     path: String, //Path or ID of the file
/// }
///```
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
}

impl FileInfo {
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
        }
    }
}

///Stores information about a directory
///
///```no_run
/// DirInfo {
///     name: String, //Name of the directory
///     path: String, //Path or ID of the directory
///     expanded: bool, //Whether the directory's contents are to be read
///     contents: Vec<FsNode>, //Contents within the directory
/// }
/// ```
#[derive(Debug, Default)]
pub struct DirInfo {
    pub name: String,
    pub path: String,
    pub expanded: bool,
    pub contents: Vec<FsNode>,
}

impl DirInfo {
    pub fn new(name: String, path: String, expanded: bool, contents: Vec<FsNode>) -> Self {
        Self {
            name,
            path,
            expanded,
            contents,
        }
    }
}

///Builds a file tree with given DirInfo struct
///
/// # Arguments
/// * `dir_info`: a reference to the DirInfo struct representing the directory
/// # Returns:
/// A `Vec<String>` where each entry is a representation of an entity within the directory
/// # Example:
///```no_run
/// let cloud_directory = g_walk("Crypt", UserToken::new_google());
///
/// let dir_tree = build_tree(cloud_directory);
/// for entity in dir_tree {
///     println!("{}", entity);    
/// }
///```
pub fn build_tree(dir_info: &DirInfo) -> Vec<String> {
    let dir_color = Color::Blue.bold();
    let mut tree: Vec<String> = Vec::new();
    let expanded_color = Color::Green.bold();
    let bracket_color = Color::White.bold();

    tree.push(format!("{}{}{}{}", 
        bracket_color.paint("[").to_string().as_str(),
        expanded_color.paint(if dir_info.expanded {"˅"} else {"˃"}),
        bracket_color.paint("]").to_string().as_str(),
        dir_color.paint(&dir_info.name).to_string().as_str()
    ));
    tree_recursion(&dir_info, String::new(), &mut tree);
    return tree;
}

///Recursively appends and walks the DirInfo contents to build a file tree
fn tree_recursion(dir_info: &DirInfo, path: String, tree: &mut Vec<String>) {
    //Force files first
    //TODO: make a config choice if folders or files first
    let (mut contents, other_content): (Vec<_>, Vec<_>) = dir_info.contents.iter()
        .partition(|n| matches!(n, FsNode::File(_)));
    contents.extend(other_content);
    
    //Character set and color
    //TODO: make a part of config and implement properly with UI
    let char_set = CharacterSet::U8_SLINE_CURVE;
    let dir_color = Color::Blue.bold();
    let expanded_color = Color::Green.bold();
    let bracket_color = Color::White.bold();

    //Set up the formatted values
    let joint = format!(" {}{}{}", char_set.joint, char_set.h_line, char_set.h_line);
    let node = format!(" {}{}{}", char_set.node, char_set.h_line, char_set.h_line);
    let vline = format!(" {}  ", char_set.v_line);

    //Iterate through contents and add them to the tree
    let contents_len = contents.len();
    for (index, entity) in contents.iter().enumerate() {
        //Determine if the current entity is last
        let is_last = index == contents_len - 1;
        //Create the prefix
        let prefix = format!("{}{}", 
            path, if is_last { &node } else { &joint }
        );

        match entity {
            FsNode::File(file) => tree.push(prefix.clone() + " " + &file.name),
            FsNode::Directory(subdir) => {
                tree.push(format!("{}{}{}{}{}", 
                    prefix.clone(), 
                    bracket_color.paint("[").to_string().as_str(),
                    expanded_color
                        .paint(if subdir.expanded {"˅"} else {"˃"})
                        .to_string().as_str(),
                    bracket_color.paint("]").to_string().as_str(),
                    dir_color.paint(&subdir.name).to_string().as_str(),
                ));

                //Recursively process expanded directories
                let sub_path = if is_last {
                    path.clone() + "    "
                } else {
                    path.clone() + &vline
                };
                if subdir.expanded {
                    tree_recursion(subdir, sub_path, tree);
                }
            },
        }
    }
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




// ///Generates a directory to convert into strings
// pub fn generate_directory(path: &PathBuf) -> anyhow::Result<DirInfo> {
//     let p = path.display().to_string();
//     //Create root
//     let mut root = DirInfo {
//         path: PathInfo::new(p.as_str()),
//         id: p,
//         expanded: true, //root is always expanded
//         contents: Vec::new(),
//     };

//     //Read contents of current directory
//     for entry in fs::read_dir(path)? {
//         let entry = entry?;
//         let p = entry.path().display().to_string();
//         let file_name = entry.file_name();
//         let file_name_str = file_name.to_string_lossy();

//         if !file_name_str.starts_with('.') && !file_name_str.starts_with("target") {
//             if path.is_dir() {
//                 root.contents.push(FileSystemEntity::Directory(DirInfo {
//                     PathInfo::new(p.as_str()),
//                     p,
//                     expanded: true, //TODO: This still shows true regardless
//                     contents: Vec::new(),
//                 }));
//             } else {
//                 root.contents
//                     .push(FileSystemEntity::File(FileInfo::new(p.as_str(), p)));
//             }
//         }
//     }
//     Ok(root)
// }

/// takes in a path, and recursively walks the subdirectories and returns a vec<pathbuf>
pub fn walk_directory(path_in: &str) -> Result<Vec<PathBuf>> {
    let path = match path_in.is_empty() {
        true => std::env::current_dir()?,
        false => get_full_file_path(path_in)?,
    };

    let walker = WalkDir::new(path).into_iter();
    let mut pathlist: Vec<PathBuf> = Vec::new();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();
        // we only want to save paths that are towards a file.
        if entry.path().display().to_string().find('.').is_some() {
            pathlist.push(PathBuf::from(entry.path().display().to_string()));
        }
    }
    Ok(pathlist)
}

/// takes in a path, and recursively walks the subdirectories and returns a vec<pathbuf>
pub fn walk_paths(path_in: &str) -> Result<Vec<PathInfo>> {
    let path = match path_in.is_empty() {
        true => std::env::current_dir()?,
        false => get_full_file_path(path_in)?,
    };

    let walker = WalkDir::new(path).into_iter();
    let mut pathlist: Vec<PathInfo> = Vec::new();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap().path().display().to_string();
        pathlist.push(PathInfo::new(entry.as_str()));
    }

    Ok(pathlist)
}

/// get full full path from a relative path
pub fn get_full_file_path(path: &str) -> Result<PathBuf> {
    Ok(dunce::canonicalize(path)?)
}

pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    let conf = config::get_config();
    entry
        .file_name()
        .to_str()
        .map(|s: &str| conf.ignore_items.contains(&s.to_string()))
        .unwrap_or(false)
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
