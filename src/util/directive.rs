use crate::{
    cloud_storage::*,
    database::crypt_keeper,
    ui::{
        cli::{self, *},
        ui_repo::CharacterSet,
    },
    util::{
        common::write_contents_to_file,
        config,
        config::{Config, ConfigTask, ItemsTask},
        encryption::{decrypt_file, encrypt_file},
        path::{generate_directory, get_full_file_path, walk_directory, walk_paths, PathInfo},
    },
};
use ansi_term::Color;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ffi::OsStr, path::PathBuf};
use tokio::runtime::Runtime;

///FileSystemEntity enum
#[derive(Debug)]
pub enum FileSystemEntity {
    File(FileInfo),
    Directory(DirInfo),
}

///Directory struct
#[derive(Debug)]
pub struct DirInfo {
    pub name: String,
    pub id: String,
    pub expanded: bool,
    pub contents: Vec<FileSystemEntity>,
}

impl DirInfo {
    pub fn new(name: String, id: String, expanded: bool, contents: Vec<FileSystemEntity>) -> Self {
        Self {
            name,
            id,
            expanded,
            contents,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub id: String,
}

///Builds a file tree with given DirInfo struct
pub fn build_tree(dir_info: &DirInfo, depth: usize, last: bool) -> Vec<String> {
    let char_set = CharacterSet::U8_SLINE_CURVE;
    let dir_color = Color::Blue.bold();
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
        tree.push(format!("{}", dir_color.paint(&dir_info.name).to_string()));
    }

    for (index, item) in dir_info.contents.iter().enumerate() {
        let is_last = index == dir_info.contents.len() - 1;
        let prefix = if is_last {
            char_set.node
        } else {
            char_set.joint
        };

        match item {
            FileSystemEntity::File(file) => {
                tree.push(format!("{}{}{} {}", indent, prefix, hline, file.name));
            }
            FileSystemEntity::Directory(subdir) => {
                tree.push(format!(
                    "{}{}{} {}",
                    indent,
                    prefix,
                    hline,
                    dir_color.paint(&subdir.name).to_string()
                ));
                let mut subtree = build_tree(subdir, depth + 1, is_last);
                tree.append(&mut subtree);
            }
        }
    }

    return tree;
}

pub fn build_tree_again(dir_info: &DirInfo, depth: usize, is_root: bool) -> Vec<String> {
    let char_set = CharacterSet::U8_SLINE_CURVE;
    let dir_color = Color::Blue.bold();
    let joint = format!("{}{}{} ", char_set.joint, char_set.h_line, char_set.h_line);
    let node = format!("{}{}{} ", char_set.node, char_set.h_line, char_set.h_line);
    let vline = format!("{}   ", char_set.v_line);

    let mut tree: Vec<String> = Vec::new();

    if is_root {
        tree.push(dir_color.paint(&dir_info.name).to_string());
    }

    for (index, entity) in dir_info.contents.iter().enumerate() {
        let is_last = index == dir_info.contents.len() - 1;
        let prefix = if is_last { &node } else { &joint };

        let mut indent = String::new();
        if depth > 0 {
            indent = vline.repeat(depth - 1) + if is_last { "    " } else { vline.as_str() };
        }

        match entity {
            FileSystemEntity::File(file) => {
                tree.push(indent.clone() + prefix + &file.name);
            }
            FileSystemEntity::Directory(subdir) => {
                tree.push(
                    indent.clone() + prefix + dir_color.paint(&subdir.name).to_string().as_str(),
                );
                tree.extend(build_tree_again(subdir, depth + 1, false));
            }
        }
    }

    return tree;
}

///Base information required for all directive calls
///
/// # Example
///```no_run
/// # use crypt_lib::util::directive::Directive;
/// let directive = Directive::new("relevant/file.path".to_string());
///```
#[derive(Debug)]
pub struct Directive {
    path: String,
}

impl Default for Directive {
    fn default() -> Self { 
        Self{ path: String::new() } 
    }
}

impl Directive {
    ///Creates a directive with the requisite path
    ///
    /// # Example
    ///```no_run
    /// # use crypt_lib::util::directive::Directive;
    /// let directive = Directive::new("relevant/file.path".to_string());
    ///```
    pub fn new(path: String) -> Self {
        Self { path }
    }

    ///Process the encryption directive
    ///
    /// # Example
    ///```no_run
    /// # use crypt_lib::util::directive::Directive;
    /// let in_place = false;
    /// let output = "desired/output/path".to_string();
    ///
    /// let directive = Directive::new("relevant/file.path".to_string());
    /// directive.encrypt(in_place, output);
    ///```
    ///TODO: implement output
    pub fn encrypt(&self, in_place: bool, _output: Option<String>) {
        //Determine if file or directory
        match PathBuf::from(&self.path).is_dir() {
            //if directory
            true => {
                // get vec of dir
                let dir = walk_directory(&self.path).expect("could not find directory!");
                // dbg!(&dir);
                for path in dir {
                    send_information(vec![format!("Encrypting file: {}", path.display())]);
                    encrypt_file(path.display().to_string().as_str(), in_place)
                }
            }
            //if file
            false => {
                encrypt_file(&self.path, in_place);
            }
        };
    }

    ///Process the decryption directive
    ///
    /// # Example
    ///```no_run
    /// # use crypt_lib::util::directive::Directive;
    /// let in_place = false;
    /// let output = "desired/output/path".to_string();
    ///
    /// let directive = Directive::new("relevant/file.path".to_string());
    /// directive.decrypt(in_place, output);
    ///```
    ///TODO: rework output for in-place
    ///TODO: implement output to just change save_location
    pub fn decrypt(&self, _in_place: bool, output: Option<String>) {
        //Determine if file or directory
        match PathBuf::from(&self.path).is_dir() {
            //if directory
            true => {
                // get vec of dir
                let dir = walk_directory(&self.path).expect("could not find directory!");
                // dbg!(&dir);
                for path in dir {
                    if path.extension().unwrap() == "crypt" {
                        send_information(vec![format!("Decrypting file: {}", path.display())]);
                        let _ =
                            decrypt_file(path.display().to_string().as_str(), output.to_owned());
                    }
                }
            }
            //if file
            false => {
                let _ = decrypt_file(&self.path, output.to_owned());
            }
        };
    }

    ///View, upload, or download files from supported cloud service
    ///
    /// # Example
    ///```no_run
    /// # use crypt_lib::util::directive::Directive;
    /// let platform = CloudPlatform::Google;
    /// let task = CloudTask::Upload;
    ///
    /// let directive = Directive::new("relevant/file.path".to_string());
    /// directive.cloud(platform, task);
    ///```
    pub fn cloud(&self, platform: oauth::CloudService, task: oauth::CloudTask) {
        //For async functions
        let runtime = Runtime::new().unwrap();

        //Actions depend on the platform
        match platform {
            //Google
            oauth::CloudService::Google => {
                //Grab user authentication token
                let user_token = oauth::UserToken::new_google();

                //Access google drive and ensure a crypt folder exists
                let crypt_folder = match runtime.block_on(drive::g_create_folder(
                    &user_token,
                    None,
                    "".to_string(),
                )) {
                    Ok(folder_id) => folder_id,
                    Err(e) => {
                        send_information(vec![format!("{}", e)]);
                        "".to_string()
                    }
                };
                // let _ = runtime.block_on(drive::g_drive_info(&user_token));
                match task {
                    oauth::CloudTask::Upload => {
                        //Track all folder ids
                        let mut folder_ids: HashMap<String, String> = HashMap::new();
                        //Fetch FileCrypts from crypt_keeper
                        let path_info = PathInfo::new(&self.path);
                        let paths =
                            walk_paths(self.path.as_str()).expect("Could not generate path(s)");
                        let paths: Vec<PathInfo> = paths
                            .into_iter()
                            .filter(|p| p.name != path_info.name)
                            .collect();

                        match path_info.is_dir {
                            true => {
                                //Create the root directory
                                folder_ids.insert(
                                    path_info.full_path.display().to_string(),
                                    runtime
                                        .block_on(drive::g_create_folder(
                                            &user_token,
                                            Some(&PathBuf::from(path_info.name.clone())),
                                            crypt_folder,
                                        ))
                                        .expect("Could not create directory in google drive"),
                                );
                                //Create all folders relative to the root directory
                                for path in paths.clone() {
                                    let parent_path = path.parent.display().to_string();
                                    let parent_id = folder_ids
                                        .get(&parent_path)
                                        .expect("Could not retrieve parent ID")
                                        .to_string();

                                    if path.is_dir {
                                        folder_ids.insert(
                                            path.full_path.display().to_string(),
                                            runtime
                                                .block_on(drive::g_create_folder(
                                                    &user_token,
                                                    Some(&PathBuf::from(path.name.clone())),
                                                    parent_id,
                                                ))
                                                .expect(
                                                    "Could not create directory in google drive",
                                                ),
                                        );
                                    }
                                }
                                //Upload every file to their respective parent directory
                                for path in paths {
                                    let parent_path = path.parent.display().to_string();
                                    let parent_id = folder_ids
                                        .get(&parent_path)
                                        .expect("Could not retrieve parent ID")
                                        .to_string();

                                    if !path.is_dir {
                                        let _ = runtime.block_on(drive::g_upload(
                                            user_token.clone(),
                                            &path.full_path.display().to_string(),
                                            parent_id,
                                        ));
                                    }
                                }
                            }
                            false => {
                                let _ = runtime.block_on(drive::g_upload(
                                    user_token,
                                    &self.path,
                                    crypt_folder,
                                ));
                            }
                        }
                    }
                    oauth::CloudTask::Download => {
                        let path_info = PathInfo::new(&self.path);
                        let paths =
                            walk_paths(self.path.as_str()).expect("Could not generate path(s)");
                        let paths: Vec<PathInfo> = paths
                            .into_iter()
                            .filter(|p| p.name != path_info.name)
                            .collect();
                        send_information(vec![format!("{:#?}", paths)]);
                        todo!()
                    }
                    oauth::CloudTask::View => {
                        let items2: DirInfo = runtime
                            .block_on(drive::g_walk(&self.path, user_token))
                            .expect("Could not view directory information");
                        send_information(build_tree_again(&items2, 0, true));
                    }
                }
            }
            oauth::CloudService::Dropbox => {
                match task {
                    oauth::CloudTask::Upload => {
                        let path = PathBuf::from(self.path.as_str());
                        match path.is_dir() {
                            true => {
                                //If folder, verify that the folder exists, create it otherwise
                            }
                            false => {}
                        }
                        //Determine if it's a file or a folder that's being uploaded
                        todo!()
                    }
                    oauth::CloudTask::Download => {
                        todo!()
                    }
                    oauth::CloudTask::View => {
                        todo!()
                    }
                }
            }
        }
    }

    ///Change configuration settings
    ///
    /// # Example
    ///```no_run
    /// # use crypt_lib::util::directive::Directive;
    /// let add_remove = ItemTask::Add;
    /// let item = "ignore.txt".to_string();
    ///
    /// let directive = Directive::new("relevant/file.path".to_string());
    /// directive.config(add_remove, item);
    ///```
    pub fn config(&self, config_task: ConfigTask) {
        let mut config = config::get_config_write();

        //Process the directive
        match config_task {
            ConfigTask::DatabasePath => match self.path.to_lowercase().as_str() {
                "" => {
                    let path = get_full_file_path(&config.database_path)
                        .expect("Error fetching database path");
                    send_information(vec![format!(
                        "Current Database Path:\n  {}",
                        path.display()
                    )]);
                }
                _ => {
                    send_information(vec![format!(
                        "{} {}",
                        "WARNING: changing your database will prevent you from decrypting existing",
                        "files until you change the path back. ARE YOU SURE? (Y/N)"
                    )]);

                    //TODO: Modify to properly handle tui/gui interactions
                    let mut s = String::new();
                    while s.to_lowercase() != *"y" || s.to_lowercase() != *"n" {
                        std::io::stdin()
                            .read_line(&mut s)
                            .expect("Did not enter a correct string");
                    }

                    if s.as_str() == "y" {
                        if PathBuf::from(&self.path).exists() {
                            config.set_database_path(&self.path);
                        } else {
                            //TODO: create path
                        }
                        config.set_database_path(&self.path);
                    }
                }
            },

            ConfigTask::IgnoreItems(option, item) => match option {
                ItemsTask::Add => config.append_ignore_items(&item),
                ItemsTask::Remove => config.remove_ignore_item(&item),
                ItemsTask::Default => {
                    let default = Config::default();
                    config.set_ignore_items(default.ignore_items);
                }
            },

            ConfigTask::Retain(value) => match config.set_retain(value) {
                true => send_information(vec![format!("Retain changed to {}", value)]),
                false => send_information(vec![format!("Error occured, please verify parameters")]),
            },

            ConfigTask::Backup(value) => match config.set_backup(value) {
                true => send_information(vec![format!("Backup changed to {}", value)]),
                false => send_information(vec![format!("Error occured, please verify parameters")]),
            },

            ConfigTask::ZstdLevel(level) => match config.set_zstd_level(level) {
                true => send_information(vec![format!("Zstd Level value changed to: {}", level)]),
                false => send_information(vec![format!("Error occured, please verify parameters")]),
            },

            ConfigTask::LoadDefault => match config.restore_default() {
                true => send_information(vec![format!("Default configuration has been restored")]),
                false => send_information(vec![format!(
                    "An error has occured attmepting to load defaults"
                )]),
            },
        };
    }
}



pub fn send_information(info: Vec<String>) {
    //TODO: Check which platform
    //CLI
    cli::print_information(info);
    //TODO: TUI
    //TODO: GUI
}
