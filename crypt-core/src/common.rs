use anyhow::{Ok, Result};
use std::any::TypeId;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use std::{
    fs::OpenOptions,
    io::Write,
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
            false => get_full_file_path(path),
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
///```ignore
/// File(FileInfo),
/// Directory(DirInfo),
///```
#[derive(Debug)]
pub enum FsNode {
    File(FileInfo),
    Directory(DirInfo),
}

impl FsNode {
    pub fn get_kind(&self) -> (Option<FileInfo>,Option<DirInfo>) {
        match self {
            FsNode::File(f) => return (Some(f), None),
            FsNode::Directory(d) => return (None, Some(d)),
        }
    }
    /// Returns FsNode name and path
    pub fn get_contents(&mut self) -> (String, String) {
        match self {
            FsNode::File(f) => return (self.name, self.path),
            _ => (),
        }
        
    }
}

///Stores information about a file
///
///```ignore
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
///```ignore
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
///```ignore
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
    
    tree.push(format!("{}", dir_color.paint(&dir_info.name).to_string().as_str()));
    tree_recursion(&dir_info, String::new(), &mut tree);
    return tree;
}

///Recursively appends and walks the DirInfo contents to build a file tree
fn tree_recursion(dir_info: &DirInfo, path: String, tree: &mut Vec<String>) {
    //The character set
    //TODO: make it so the character set is a config static variable that can be chosen by the user
    let char_set = CharacterSet::U8_SLINE_CURVE;

    //The color of the directory
    //TODO: make this a config static variable that can be chosen by the user
    //TODO: implement properly for both CLI and TUI
    let dir_color = Color::Blue.bold();

    //Set up the formatted values
    let joint = format!("{}{}{} ", char_set.joint, char_set.h_line, char_set.h_line);
    let node = format!("{}{}{} ", char_set.node, char_set.h_line, char_set.h_line);
    let vline = format!("{}   ", char_set.v_line);

    //Count the files and folders within dir_info
    let mut files: usize = 0;
    let mut folders: usize = 0;
    for entity in dir_info.contents.iter() {
        match entity {
            FsNode::File(_) => files += 1,
            FsNode::Directory(_) => folders += 1,
        }
    }

    //Process and list files first
    //TODO: find a way to do this all in one pass
    for entity in dir_info.contents.iter() {
        let is_last = folders < 1 && files == 1;
        let prefix = if is_last { &node } else { &joint };

        match entity {
            FsNode::File(file) => {
                tree.push(path.clone() + prefix + &file.name);
            },
            FsNode::Directory(_) => (),
        }
        if files > 0 {
            files -= 1;
        }
    }

    //Process and list folders last
    //TODO: find a way to do this all in one pass
    for entity in dir_info.contents.iter() {
        let is_last = folders <= 1;
        let prefix = if is_last { &node } else { &joint };

        match entity {
            FsNode::File(_) => (),
            FsNode::Directory(subdir) => {
                tree.push(format!("{}{}", 
                    path.clone() + prefix.as_str(), 
                    dir_color.paint(&subdir.name).to_string().as_str()
                ));
                let sub_path = if is_last {path.clone() + "    "} else {path.clone() + &vline};
                //Recursively process expanded directories
                if subdir.expanded {
                    tree_recursion(subdir, sub_path, tree);
                }
            },
        }
        if folders > 0 {
            folders -= 1;
        }
    }
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
        false => get_full_file_path(path_in),
    };
    dbg!(&path);
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
pub fn walk_paths(path_in: &str) -> Vec<PathInfo> {
    let path = match path_in.is_empty() {
        true => std::env::current_dir().unwrap(),
        false => get_full_file_path(path_in),
    };
    let walker = WalkDir::new(path).into_iter();
    let mut pathlist: Vec<PathInfo> = Vec::new();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap().path().display().to_string();
        pathlist.push(PathInfo::new(entry.as_str()));
    }

    pathlist
}


/// get full full path from a relative path
pub fn get_full_file_path(path: &str) -> PathBuf {
    let canonicalize = dunce::canonicalize(path);
    match canonicalize {
        core::result::Result::Ok(c) => return c,
        Err(_) => PathBuf::from(path),
    }
}

pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    let conf = config::get_config();
    let mut b: bool = false;
    entry
        .file_name()
        .to_str()
        .map(|s: &str| {
            conf.ignore_items.into_iter().for_each(|item| {
                b = s.to_string().contains(&item) || s.starts_with('.');
            }) 
        });
        b
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_directory() {
        let path = "../test_folder/";
        let res = walk_directory(path).unwrap();
        assert_eq!(res[0].file_name().unwrap().to_str().unwrap(),"file1.txt");
    }
}
