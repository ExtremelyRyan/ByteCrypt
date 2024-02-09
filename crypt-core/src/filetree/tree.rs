use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum FileTree {
    DirNode(Directory),
    FileNode(File),
    LinkNode(Symlink),
}

#[derive(Debug)]
pub struct Directory {
    pub name: String,
    pub entries: Vec<FileTree>,
}

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub metadata: fs::Metadata,
}

#[derive(Debug)]
pub struct Symlink {
    pub name: String,
    pub target: String,
    pub metadata: fs::Metadata,
}

pub fn is_not_hidden(name: &str) -> bool {
    return !name.starts_with('.');
}

pub fn sort_by_name(a: &fs::DirEntry, b: &fs::DirEntry) -> Ordering {
    let a_name: String =
        a.path().file_name().unwrap().to_str().unwrap().into();
    let b_name: String =
        b.path().file_name().unwrap().to_str().unwrap().into();
    a_name.cmp(&b_name)
}

pub fn dir_walk(
    root: &PathBuf,
    filter: fn(name: &str) -> bool,
    compare: fn(a: &fs::DirEntry, b: &fs::DirEntry) -> Ordering,
) -> io::Result<Directory> {
    let mut entries: Vec<fs::DirEntry> = fs::read_dir(root)?
        .filter_map(|result| result.ok())
        .collect();
    entries.sort_by(compare);
    let mut directory: Vec<FileTree> = Vec::with_capacity(entries.len());
    for e in entries {
        let path = e.path();
        let name: String = path.file_name().unwrap().to_str().unwrap().into();
        if !filter(&name) {
            continue;
        };
        let metadata = fs::metadata(&path).unwrap();
        let node = match path {
            path if path.is_dir() => {
                FileTree::DirNode(dir_walk(&root.join(name), filter, compare)?)
            }
            path if path.is_symlink() => FileTree::LinkNode(Symlink {
                name,
                target: fs::read_link(path)
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                metadata,
            }),
            path if path.is_file() => {
                FileTree::FileNode(File { name, metadata })
            }
            _ => unreachable!(),
        };
        directory.push(node);
    }
    let name = root
        .file_name()
        .unwrap_or(OsStr::new("."))
        .to_str()
        .unwrap()
        .into();
    Ok(Directory {
        name,
        entries: directory,
    })
}
