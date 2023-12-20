use anyhow::{Ok, Result};
use std::{fs::{self, File}, path::PathBuf};
use walkdir::WalkDir;

use crate::{config, common::{PathInfo, FileSystemEntity, FileInfo, DirInfo}};



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
                    .push(FileSystemEntity::File(FileInfo::new(p.as_str())));
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
        .map(|s: &str| conf.ignore_items.contains(&s.to_string()))
        .unwrap_or(false)
}
