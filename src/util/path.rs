use anyhow::{Ok, Result};
use std::{env, fs, path::{PathBuf, Path}};
use walkdir::WalkDir;

///Directory struct
pub struct Directory {
    pub path: PathBuf,
    pub expanded: bool,
    pub contents: Vec<FileSystemEntity>,
}

///FileSystemEntity enum
pub enum FileSystemEntity {
    File(PathBuf),
    Directory(Directory),
}

///Generates a directory to convert into strings
pub fn generate_directory(/*base_path: &str,*/ current_directory: &PathBuf) -> anyhow::Result<Directory> {
    //Create root
    let mut root = Directory {
        path: current_directory.clone(),
        expanded: true, //root is always expanded
        contents: Vec::new(),
    };

    //Read contents of current directory
    for entry in fs::read_dir(current_directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if !file_name_str.starts_with('.') && !file_name_str.starts_with("target") {
            if path.is_dir() {
                root.contents.push(FileSystemEntity::Directory(Directory {
                    path,
                    expanded: true,
                    contents: Vec::new(),
                }));
            } else {
                root.contents.push(FileSystemEntity::File(path));
            }
        }
    }
    return Ok(root);
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

/// get full full path from a relative path
pub fn get_full_file_path(path: &str) -> Result<PathBuf> {
    Ok(dunce::canonicalize(path)?)
}

pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    //? add configurable hidden extensions here
    //? something like: vec!["target", "another", "and another"], etc.
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') || s.starts_with("target"))
        .unwrap_or(false)
}  