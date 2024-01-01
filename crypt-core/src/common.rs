use anyhow::{Ok, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs::OpenOptions, io::Write};
use walkdir::WalkDir;

use crate::config;
use crate::ui_repo::CharacterSet;
use ansi_term::Color;

/// given a path, dissect and return a struct containing the full path, is_dir, parent path, and name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone)]
pub enum FsNode {
    File(FileInfo),
    Directory(DirInfo),
}

impl FsNode {
    pub fn is_dir(&self) -> bool {
        match self {
            FsNode::File(_) => false,
            FsNode::Directory(_) => true,
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            FsNode::File(f) => f.name.as_str(),
            FsNode::Directory(d) => d.name.as_str(),
        }
    }

    pub fn get_path_str(&self) -> &str {
        match self {
            FsNode::File(f) => f.path.as_str(),
            FsNode::Directory(d) => d.path.as_str(),
        }
    }

    pub fn get_path_string(&self) -> String {
        match self {
            FsNode::File(f) => f.path.clone(),
            FsNode::Directory(d) => d.path.clone(),
        }
    }

    pub fn get_pathbuf(&self) -> Option<PathBuf> {
        let path_str = self.get_path_str();

        match Path::new(path_str).exists() {
            true => Some(PathBuf::from(path_str)),
            false => None,
        }
    }

    pub fn get_expanded(&self) -> Option<bool> {
        match self {
            FsNode::File(_) => None,
            FsNode::Directory(d) => Some(d.expanded),
        }
    }

    pub fn get_contents(&self) -> Option<Vec<FsNode>> {
        match self {
            FsNode::File(_) => None,
            FsNode::Directory(d) => Some(d.contents.clone()),
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
        Self { name, path }
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
#[derive(Debug, Default, Clone)]
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
    let expanded_color = Color::Green.bold();
    let bracket_color = Color::White.bold();

    tree.push(format!(
        "{}{}{}{}",
        bracket_color.paint("[").to_string().as_str(),
        expanded_color.paint(if dir_info.expanded { "˅" } else { "˃" }),
        bracket_color.paint("]").to_string().as_str(),
        dir_color.paint(&dir_info.name).to_string().as_str()
    ));
    tree_recursion(dir_info, String::new(), &mut tree);
    tree
}

///Recursively appends and walks the DirInfo contents to build a file tree
fn tree_recursion(dir_info: &DirInfo, path: String, tree: &mut Vec<String>) {
    //Force files first
    //TODO: make a config choice if folders or files first
    let (mut contents, other_content): (Vec<_>, Vec<_>) = dir_info
        .contents
        .iter()
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
        let prefix = format!("{}{}", path, if is_last { &node } else { &joint });

        match entity {
            FsNode::File(file) => tree.push(prefix.clone() + " " + &file.name),
            FsNode::Directory(subdir) => {
                tree.push(format!(
                    "{}{}{}{}{}",
                    prefix.clone(),
                    bracket_color.paint("[").to_string().as_str(),
                    expanded_color
                        .paint(if subdir.expanded { "˅" } else { "˃" })
                        .to_string()
                        .as_str(),
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
            }
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
        core::result::Result::Ok(c) => c,
        Err(_) => PathBuf::from(path),
    }
}

pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    let conf = config::get_config();
    let mut b: bool = false;
    if let Some(s) = entry.file_name().to_str() {
        conf.ignore_items.into_iter().for_each(|item| {
            // TODO: make this better ------------------v
            b = s.to_string().contains(&item) || s.starts_with('.');
        })
    };
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_directory() {
        let path = ".";
        let res = walk_directory(path).unwrap();
        assert_eq!(
            res[0].file_name().unwrap().to_str().unwrap(),
            "encryption_benchmark.rs"
        );
    }
}
