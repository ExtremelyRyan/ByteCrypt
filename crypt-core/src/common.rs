use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};
use walkdir::WalkDir;

use crate::{config, error, prelude::*};
use ansi_term::Color;
use serde_json::Value;

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
    /// name of the directory
    pub name: String,
    /// path or ID of the directory
    pub path: String,
    /// boolean deciding if the contents are displayed
    pub expanded: bool,
    /// contents within the directory
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

/// Verifies the existence of a file or directory at the specified path.
///
/// This function checks whether the given path exists in the filesystem.
/// It can be used to verify the existence of a file or directory before
/// performing further operations.
///
/// # Arguments
/// * `path` - A type that can be converted to a `&Path`. This includes
///           types like `&str` and `String`.
///
/// # Returns
/// Returns `true` if the path exists, `false` otherwise.
///
/// # Examples
/// ```rust ignore
/// use std::path::Path;
///
/// let existing_path = "path/to/existing/file.txt";
/// let non_existing_path = "path/to/non/existing/file.txt";
///
/// assert_eq!(verify_path(existing_path), true);
/// assert_eq!(verify_path(non_existing_path), false);
/// ```
pub fn verify_path(path: &impl AsRef<Path>) -> bool {
    let as_ref = path.as_ref();
    as_ref.exists()
}

/// Retrieves the relative path from the current working directory to the specified target path.
///
/// This function takes a target path, resolves its full path by joining it with
/// the current working directory, and then determines the relative path from the
/// current working directory to the target path. The result is returned as a `PathBuf`.
///
/// # Arguments
///
/// * `target_path` - A type that can be converted to a `&Path`. This includes
///                  types like `&str` and `String`.
///
/// # Returns
///
/// Returns a `Result` containing a `PathBuf` representing the relative path from
/// the current working directory to the target path. If an error occurs during
/// the process (e.g., failure to retrieve the current working directory or join
/// paths), an `Err` variant with a `std::io::Error` is returned.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// # use crate::crypt_core::common::get_path_diff;
///
/// fn main() {
///     match get_path_diff(None, &PathBuf::from("path/to/target")) {
///         Ok(relative_path) => {
///             println!("Relative Path: {:?}", relative_path);
///         }
///         Err(error) => {
///             eprintln!("Error: {}", error);
///         }
///     }
/// }
/// ```
///
/// # Notes
///
/// - If the target path is not found, an empty `PathBuf` is returned.
/// - The function prints debug information about the current working directory,
///   full path, and relative path to aid in understanding the process.
///
/// # Errors
///
/// This function returns an `Err` variant if an error occurs during the process.
/// Possible errors include failure to retrieve the current working directory or
/// failure to join paths.
///
pub fn get_path_diff<T>(root: Option<&T>, target_path: &T) -> Result<PathBuf>
where
    T: AsRef<Path>,
{
    let target_path = target_path.as_ref();

    // if root directory is Some, get contents.
    // otherwise, get the user's current working directory.
    let current_dir: PathBuf = match root {
        Some(dir) => dir.as_ref().to_owned(),
        None => std::env::current_dir()?,
    };

    // Resolve the full path of the target path
    let full_path = current_dir.join(target_path);

    // Get the relative path from the current directory to the target path
    let relative_path = match full_path.strip_prefix(&current_dir) {
        Ok(c) => c,
        Err(_) => Path::new(""),
    };

    Ok(relative_path.to_owned())
}

///Builds a file tree with given DirInfo struct
///
/// # Arguments
/// * `dir_info`: a reference to the DirInfo struct representing the directory
/// # Returns:
/// A `Vec<String>` where each entry is a representation of an entity within the directory
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

/// Recursively appends and walks the contents of a `DirInfo` structure to build a file tree.
///
/// # Arguments
///
/// * `dir_info` - A `DirInfo` structure representing a directory and its contents.
/// * `path` - The current path in the file tree. Used to construct prefixes for visual representation.
/// * `tree` - A mutable vector to store the lines of the resulting file tree.
///
/// # Tree Visualization
///
/// - The file tree is constructed with files listed first, followed by directories.
/// - The expansion state of directories is indicated by arrows (˅ for expanded, ˃ for collapsed).
/// - The function uses a character set and color configuration for visual appeal.
///
/// # Prefix Formatting
///
/// - The lines of the tree are formatted with proper characters for junctions, nodes, and vertical lines.
/// - The `path` parameter is used to construct prefixes for each line in the tree.
///
/// # Configuration Options
///
/// - TODO: Consider adding a configuration choice for ordering folders or files first.
/// - TODO: Implement a more flexible configuration system for character sets and colors.
/// - TODO: Improve the handling of UI-related configurations.
///
/// # Notes
///
/// - Directories are processed recursively, and expanded directories lead to deeper levels in the tree.
/// - The `tree` vector is mutated to store each line of the resulting file tree.
///
/// # TODO
///
/// - Consider adding a configuration choice for ordering folders or files first.
/// - Implement a more flexible configuration system for character sets and colors.
/// - Improve the handling of UI-related configurations.
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
    let dir_color = Color::Blue.bold();
    let expanded_color = Color::Green.bold();
    let bracket_color = Color::White.bold();

    //Set up the formatted values
    let joint = format!(" {}{}{}", '├', '─', '─');

    let node = format!(" {}{}{}", '╰', '─', '─');

    let vline = format!(" {}  ", '│');

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

// todo: wrap this in result
/// read file, and return values within a Vector of u8.
pub fn get_vec_file_bytes(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_default()
}

pub fn get_file_contents<T: AsRef<Path>>(path: T) -> Result<Vec<u8>> {
    return std::fs::read(path).map_err(Error::IoError);
}

/// Writes the contents of a `Vec<u8>` to a file.
///
/// # Arguments
///
/// * `file` - The path to the file to be written.
/// * `contents` - The data to be written to the file.
///
/// # Returns
///
/// Returns a `Result` indicating whether the write operation was successful.
///
pub fn write_contents_to_file<T: AsRef<Path>>(file: T, contents: Vec<u8>) -> Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .truncate(true)
        .open(file.as_ref())?;
    f.write_all(contents.as_slice())?;
    f.flush()?;
    Ok(())
}

/// Performs a system command to get user home path.
/// if system is a windows machine, performs a powershell call. Otherwise, we assume it is linux
/// and
pub fn get_config_folder() -> PathBuf {
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
    path.push("crypt_config");

    if !path.exists() {
        _ = std::fs::create_dir(&path);
    }

    path
}

/// performs a process command to query user profile.
/// if on windows, we use `cmd`. If on Linux, we use `sh`
/// returns a `PathBuf` of the home path with "crypt"
/// appended to the end of the path if query was sucessful.
///
/// # Example
/// assuming user profile name is ryan
/// ```rust ignore
/// let path = get_crypt_folder();
/// // for windows
/// assert_eq(path, "C:\\users\\ryan\\crypt");
/// // for linux
/// assert_eq(path, "~/home/ryan/crypt");
/// ```
/// # Panics
///
/// function can panic if either the process fails,
/// or the conversion from `Vec<u8>` to String fails.
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

/// Performs a command to query the device hostname.
///
/// If the target operating system is Windows, the function uses the command prompt (`cmd`).
/// Otherwise, it uses the shell (`sh`).
///
/// # Panics
///
/// This function may panic under the following conditions:
///
/// - The process execution fails.
/// - The conversion from `Vec<u8>` to `String` fails.
///
/// # Returns
///
/// Returns the hostname as a `String` if the query is successful.
///
/// # Examples
///
/// ```rust ignore
/// # use crate::get_machine_name;
/// let hostname = get_machine_name();
/// println!("Hostname: {}", hostname);
/// ```
pub fn get_machine_name() -> String {
    let name = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "hostname"])
            .output()
            .expect("failed to get hostname")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("hostname")
            .output()
            .expect("failed to get pc name")
    };

    String::from_utf8(name.stdout)
        .expect("converting stdout failed")
        .trim()
        .to_string()
}

pub fn get_filenames_from_subdirectories<T: AsRef<Path>>(
    dir_path: T,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let walker = WalkDir::new(&dir_path).into_iter();

    let (filenames, folders): (Vec<_>, Vec<_>) = walker
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file() || entry.path().is_dir())
        .partition(|entry| entry.path().is_file());

    let filenames: Vec<_> = filenames
        .into_iter()
        .map(|entry| entry.path().to_path_buf())
        .collect();
    let folders: Vec<_> = folders
        .into_iter()
        .map(|entry| entry.path().to_path_buf())
        .collect();

    Ok((filenames, folders))
}

/// Displays a menu for choosing files and folders from the users `crypt` folder.
///
/// # Arguments
///
/// * `item`: A string representing the filename or part of it to filter the list.
///
/// # Returns
///
/// Returns a `PathBuf` representing the user's selection. Returns an empty `PathBuf` if the user chooses to abort.
///
pub fn chooser(item: &str) -> Result<PathBuf> {
    let (mut files, mut folders) = walk_crypt_folder().unwrap_or_else(|_| (Vec::new(), Vec::new()));

    if files.is_empty() {
        return Err(Error::CommonError(error::CommonError::CryptFolderIsEmpty));
    }

    files.sort();
    folders.sort();

    let mut count = 1;

    if !item.is_empty() {
        // Filter files based on item
        files.retain(|p| {
            p.file_stem()
                .map(|stem| stem.to_ascii_lowercase().to_string_lossy().contains(item))
                .unwrap_or(false)
                || p.file_name()
                    .map(|name| name.to_ascii_lowercase() == item)
                    .unwrap_or(false)
        });

        // Filter folders based on item
        folders.retain(|p| {
            p.file_stem()
                .map(|stem| stem.to_ascii_lowercase().to_string_lossy().contains(item))
                .unwrap_or(false)
                || p.file_name()
                    .map(|name| name.to_ascii_lowercase() == item)
                    .unwrap_or(false)
        });
    }

    println!("\nplease choose from the following: (or 0 to abort)");
    println!("{: <3} {: <45} {: <14}", "#", "files", "last modified");
    println!("----------------------------------------------------------------");
    for item in &files {
        let item_str = item.to_string_lossy();

        let partal_path = if let Some(found) = item_str.find(r#"\crypt"#) {
            // split at "found" + 6 to get rid of '\crypt' to save space
            item_str.split_at(found + 6).1
        } else {
            &item_str
        };

        let cropped_str = Path::new(partal_path).to_string_lossy().to_string();

        // since our main width is 45 characters, crop path so we look nice and neat.
        let display_path = match cropped_str.len() > 45 {
            true => &cropped_str[cropped_str.len() - 45..],
            false => cropped_str.as_ref(),
        };

        println!(
            "{: <3} {: <45} {: <14}",
            count,
            display_path,
            get_sys_time_timestamp(item.metadata().unwrap().modified().unwrap())
        );

        count += 1;
    }

    if !folders.is_empty() {
        println!("----------------------------------------------------------------\n");
        println!("{: <3} {: <45} ", "#", "folders",);
        println!("----------------------------------------------------------------");

        for i in folders {
            if i.display().to_string() == std::path::MAIN_SEPARATOR_STR {
                continue;
            }
            let item_str = i.to_string_lossy();

            let partal_path = if let Some(found) = item_str.find(r#"\crypt"#) {
                // split at "found" + 6 to get rid of '\crypt' to save space
                item_str.split_at(found).1
            } else {
                &item_str
            };

            // convert to string for easier length checking and cropping if needed.
            let cropped_str = Path::new(partal_path).to_string_lossy().to_string();

            // since our main width is 45 characters, crop path so we look nice and neat.
            let display_path = match cropped_str.len() > 45 {
                true => &cropped_str[cropped_str.len() - 45..],
                false => cropped_str.as_ref(),
            };
            println!("{: <3} {: <45}", count, display_path);

            // add folder to files vector for easier picking later.
            files.push(i.to_path_buf());

            count += 1;
        }
        println!("----------------------------------------------------------------");
    }
    // Get choice from user
    loop {
        let mut number = String::new();
        if io::stdin().read_line(&mut number).is_err() {
            println!("Error reading input. Please try again.");
            continue;
        }

        let num: usize = number.trim().parse().unwrap_or_default();
        if num == 0 {
            return Err(Error::CommonError(error::CommonError::UserAbort));
        }
        if num > files.len() {
            println!("invalid selection. please try again.");
            continue;
        }

        return Ok(files[num - 1].to_owned());
    }
}

/// Converts a `SystemTime` into a formatted "date : time" string.
///
/// This function takes a `SystemTime` value and converts it into a human-readable
/// string representing the date and time in the "MM/DD/YY HH:MM" format.
///
/// # Arguments
///
/// * `ts`: A `SystemTime` value to be converted.
///
/// # Returns
///
/// Returns a `String` representing the formatted date and time.
///
/// # Examples
///
/// ```rust ignore
/// use std::time::SystemTime;
/// use crate::get_sys_time_timestamp;
///
/// let current_time = SystemTime::now();
/// let formatted_time = get_sys_time_timestamp(current_time);
/// println!("Formatted Time: {}", formatted_time);
/// ```
fn get_sys_time_timestamp(ts: SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Utc> = ts.into();
    dt.format("%m/%d/%y %H:%M").to_string()
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

/// Parse JSON Token from File
///
/// This function reads a JSON file containing Google OAuth configuration, extracts the necessary
/// information, and sets environment variables accordingly.
///
/// # Errors
///
/// This function returns a `Result<(), Error>` where `Error` is the type for any error that occurred
/// during the parsing process. This includes file reading errors, JSON parsing errors, and any other
/// related issues.
///
/// # Panics
///
/// This function may panic if the JSON file format does not match the expected structure.
/// It's recommended to ensure the JSON file is properly formatted and contains the required fields.
///
/// # Environment Variables
///
/// This function sets the following environment variables based on the JSON content:
/// - `GOOGLE_CLIENT_ID`: Google OAuth client ID.
/// - `GOOGLE_CLIENT_SECRET`: Google OAuth client secret.
///
pub fn parse_json_token() -> Result<()> {
    let mut config_path = get_config_folder();
    config_path.push("google.json");

    // Open the file in read-only mode with buffer.
    let file = File::open(config_path)?;

    // Read the JSON contents of the file as an instance of `User`.
    let v: Value = serde_json::from_reader(BufReader::new(file))?;

    let mut client: String = v["web"]["client_id"].to_string();
    client = client.replace(&['\"'][..], "");
    let mut secret: String = v["web"]["client_secret"].to_string();
    secret = secret.replace(&['\"'][..], "");

    std::env::set_var("GOOGLE_CLIENT_ID", client);
    std::env::set_var("GOOGLE_CLIENT_SECRET", secret);
    Ok(())
}

/// Called to print any information passed.
///
/// # Arguments
///
/// * `info` - An iterable collection of items that implement the `Display` trait.
///
/// # Examples
///
/// ```rust
/// # use crypt_core::common::print_information;
///
/// let info = vec!["Item 1", "Item 2", "Item 3"];
/// print_information(info);
/// ```
pub fn print_information<T>(info: T)
where
    T: IntoIterator,
    T::Item: Display,
{
    for item in info {
        println!("{}", item);
    }
}

pub fn send_information(info: Vec<String>) {
    //CLI
    print_information(info);
}

/// Takes in a path, and recursively walks the subdirectories and returns a `Vec<PathBuf>`
/// The `filter_directories` parameter determines whether to filter entries based on the presence of a dot ('.')
/// # Examples
/// ```rust
///  # use crate::crypt_core::common::walk_directory;
/// // setting `filter_directories` to `false` includes directories.
/// let res = walk_directory("test_folder", false);
/// println!("{:#?}", res);
///
/// // setting `filter_directories` to `true` excludes directories.
/// let res = walk_directory("test_folder", true);
/// println!("{:#?}", res);
/// ```
pub fn walk_directory<T: AsRef<Path>>(
    path_in: T,
    filter_directories: bool,
) -> Result<Vec<PathBuf>> {
    let path_in = path_in.as_ref();
    let path = match path_in.to_string_lossy().is_empty() {
        true => std::env::current_dir()?,
        false => get_full_file_path(path_in),
    };

    let walker = WalkDir::new(path).into_iter();
    let mut pathlist: Vec<PathBuf> = Vec::new();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry?;

        if !filter_directories || entry.path().display().to_string().find('.').is_some() {
            pathlist.push(PathBuf::from(entry.path().display().to_string()));
        }
    }

    Ok(pathlist)
}

/// Recursively walks the subdirectories of the crypt folder and returns a `Vec<PathBuf>`.
///
/// # Returns
///
/// Returns a Result containing a `Vec<PathBuf>` with paths to files within the crypt folder,
/// excluding certain folders such as "logs" and "decrypted". If an error occurs during the
/// walking process, an Err variant is returned with an associated error message.
///
/// # Errors
///
/// This function may return an error if there are issues with walking the directories or
/// filtering entries.
///
/// # Examples
///
/// ```rust ignore  
/// match walk_crypt_folder() {
///     Ok(paths) => {
///         for path in paths {
///             println!("Found file: {}", path.display());
///         }
///     }
///     Err(err) => eprintln!("Error: {}", err),
/// }
/// ```
pub fn walk_crypt_folder() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let crypt_folder = get_crypt_folder();

    // folders to avoid
    let log_folder = Path::new(&crypt_folder).join("logs");
    let decrypted_folder = Path::new(&crypt_folder).join("decrypted");

    let walker = WalkDir::new(crypt_folder).into_iter();

    let (filenames, folders): (Vec<_>, Vec<_>) = walker
        .filter_entry(|e| {
            !is_hidden(e)
                && !e.path().starts_with(&log_folder)
                && !e.path().starts_with(&decrypted_folder)
        })
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.path().is_file() || entry.path().is_dir() {
                Some(entry.path().to_owned())
            } else {
                None
            }
        })
        .partition(|entry| entry.is_file());
    Ok((filenames, folders))
}

/// takes in a path, and recursively walks the subdirectories and returns a `vec<pathbuf>`
pub fn walk_paths<T: AsRef<str>>(path_in: T) -> Vec<PathInfo> {
    let path = match path_in.as_ref().is_empty() {
        true => std::env::current_dir().unwrap_or_else(|err| {
            eprintln!("Error getting current directory: {}", err);
            PathBuf::new()
        }),
        false => get_full_file_path(path_in.as_ref()),
    };

    if !path.exists() {
        eprintln!("Path does not exist: {:?}", path);
        return Vec::new();
    }

    let walker = WalkDir::new(path).into_iter();
    let mut pathlist: Vec<PathInfo> = Vec::new();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        if let Ok(entry) = entry {
            let entry_path = entry.path().display().to_string();
            pathlist.push(PathInfo::new(entry_path.as_str()));
        } else {
            eprintln!("Error processing directory entry");
        }
    }

    pathlist
}

/// get full path from a relative path
pub fn get_full_file_path<T: AsRef<Path>>(path: T) -> PathBuf {
    let canonicalize = dunce::canonicalize(path.as_ref());
    match canonicalize {
        Ok(c) => c,
        Err(_) => PathBuf::from(path.as_ref()),
    }
}

/// Checks whether a `DirEntry` should be considered hidden based on the configured
/// items to ignore in the file system.
///
/// This function examines the file name of a `DirEntry` and determines whether it
/// should be considered hidden according to the configured items to ignore. The
/// configuration is obtained using `config::get_config()`.
///
/// # Arguments
///
/// * `entry`: A reference to a `DirEntry` representing a file or directory entry.
///
/// # Returns
///
/// Returns `true` if the file should be considered hidden, and `false` otherwise.
///
/// # Examples
///
/// ``` rust ignore
/// use walkdir::DirEntry;
/// use crate::is_hidden;
///
/// // Assuming you have a DirEntry, e.g., obtained during directory traversal
/// let dir_entry = /* ... */;
///
/// if is_hidden(&dir_entry) {
///     println!("The file is hidden.");
/// } else {
///     println!("The file is not hidden.");
/// }
/// ```
///
/// # Note
///
/// - If the file name is not a valid UTF-8 string, it is considered hidden.
/// - The configuration, obtained through `config::get_config()`, specifies items
///   to ignore, and any file name containing or starting with these items is
///   considered hidden.
pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    let conf = config::get_config();

    if let Some(s) = entry.file_name().to_str() {
        // Early return if the file name is not a valid UTF-8 string
        if s.is_empty() {
            return true;
        }

        // TODO: change to support including hidden files?
        // Use the `any` method for a more concise check
        return conf
            .ignore_items
            .iter()
            .any(|item| s.contains(item) || s.starts_with('.'));
    }

    true // Return true if the file name is not a valid UTF-8 string
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // works locally, but for some reason fails in the CI test!
    #[ignore]
    fn test_walk_directory() {
        let path = ".";
        let res = walk_directory(path, false).unwrap();
        assert_eq!(
            res[0].file_name().unwrap().to_str().unwrap(),
            "encryption_benchmark.rs"
        );
    }
}
