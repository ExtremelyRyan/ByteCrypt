use anyhow::{Ok, Result};
use std::{fs, path::PathBuf};
use walkdir::WalkDir;
use crate::util::config;


/// given a path, dissect and return a struct containing the full path, is_dir, parent path, and name.
///
/// # Example
/// <b>assuming current working directory is `C:/test/folder1/`</b>
/// ```no_run
/// # use crypt_lib::util::encryption::get_file_info;
/// # use std::path::PathBuf;
/// let path = "file.txt";
/// let info = PathInfo::new(path);
/// assert_eq!(info.full_path, PathBuf::from("C:\\test\\folder1\\file.txt"));
/// assert_eq!(info.is_dir, PathBuff::from("C\\test\\folder1\\file.txt").is_dir());
/// assert_eq!(info.parent, PathBuf::from("C:\\test\\folder1"));
/// assert_eq!(info.name, "file.txt");
/// ```
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

///Directory struct
#[derive(Debug)]
pub struct DirInfo {
    // pub path: PathBuf,
    pub path: PathInfo,
    pub expanded: bool,
    pub contents: Vec<FileSystemEntity>,
}

///FileSystemEntity enum
#[derive(Debug)]
pub enum FileSystemEntity {
    File(PathInfo),
    Directory(DirInfo),
}

///Generates a directory to convert into strings
pub fn generate_directory(path: &PathBuf) -> anyhow::Result<DirInfo> {
    let p = path.display().to_string();
    //Create root
    let mut root = DirInfo {
        path: PathInfo::new(p.as_str()),
        expanded: true, //root is always expanded
        contents: Vec::new(),
    };

    //Read contents of current directory
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path().display().to_string();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if !file_name_str.starts_with('.') && !file_name_str.starts_with("target") {
            if path.is_dir() {
                root.contents.push(FileSystemEntity::Directory(DirInfo {
                    path: PathInfo::new(p.as_str()),
                    expanded: true, //TODO: This still shows true regardless
                    contents: Vec::new(),
                }));
            } else {
                root.contents
                    .push(FileSystemEntity::File(PathInfo::new(p.as_str())));
            }
        }
    }
    Ok(root)
}

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
        .map(|s| conf.ignore_items.contains(&s.to_string()))
        .unwrap_or(false)
}
