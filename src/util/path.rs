use anyhow::{Ok, Result};
use glob::glob;
use std::{env, fs, path::PathBuf};
use walkdir::WalkDir;

pub fn walk_directory(path_in: &str) -> Result<Vec<String>> {
    
    let path = match path_in.is_empty() {
        true => std::env::current_dir()?,
        false => get_full_file_path(path_in)?,
    };

    let walker = WalkDir::new(path).into_iter();
    let mut pathlist: Vec<String> = Vec::new();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();
        println!("{}", entry.path().display());
        // we only want to save paths that are towards a file.
        if entry.path().display().to_string().find(".").is_some() {
            pathlist.push(entry.path().display().to_string());
        }
    }
    Ok(pathlist)
}

pub fn get_full_file_path(path: &str) -> Result<PathBuf> {
    Ok(dunce::canonicalize(path)?)
}

pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with(".") || s.starts_with("target"))
        .unwrap_or(false)
}

pub fn walk_dir() -> anyhow::Result<()> {
    let mut entries = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    // The order in which `read_dir` returns entries is not guaranteed. If reproducible
    // ordering is required the entries should be explicitly sorted.
    entries.sort();

    for e in entries {
        println!("{:?}", e);
    }

    // The entries have now been sorted by their path.
    Ok(())
}

pub fn walk() -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    println!(
        "Entries modified in the last 24 hours in {:?}:",
        current_dir
    );

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        let metadata = fs::metadata(&path)?;
        let last_modified = metadata.modified()?.elapsed()?.as_secs();

        if last_modified < 24 * 3600 && metadata.is_file() {
            println!(
                "Last modified: {:?} seconds, is read only: {:?}, size: {:?} bytes, filename: {:?}",
                last_modified,
                metadata.permissions().readonly(),
                metadata.len(),
                path.file_name().ok_or("No filename").unwrap()
            );
        }
    }
    Ok(())
}

pub fn find_file_with_name() -> Result<()> {
    for entry in WalkDir::new(".")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = entry.file_name().to_string_lossy();
        let sec = entry.metadata()?.modified()?;

        if f_name.ends_with(".rs") && sec.elapsed()?.as_secs() < 86400 {
            println!("{}", f_name);
        }
    }
    Ok(())
}
