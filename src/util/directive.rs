use crate::{
    cloud_storage::*,
    database::crypt_keeper,
    ui::cli::*,
    util::{
        common::write_contents_to_file,
        config::Config,
        encryption::{decrypt_file, encrypt_file},
        path::{generate_directory, get_full_file_path, walk_directory, walk_paths, PathInfo},
    },
};
use std::{ffi::OsStr, path::PathBuf, collections::HashMap};
use tokio::runtime::Runtime;

///Information required for an encryption command
#[derive(Debug)]
pub struct EncryptInfo {
    pub path: String,
    pub in_place: bool,
    pub config: Config,
}

///Information required for a deryption command
#[derive(Debug)]
pub struct DecryptInfo {
    pub path: String,
    pub output: Option<String>,
    pub config: Config,
}

///Supported cloud platforms
#[derive(Debug)]
pub enum CloudPlatform {
    Google,
    DropBox,
}

///Supported tasks for cloud platforms
#[derive(Debug)]
pub enum CloudTask {
    Upload,
    Download,
    View,
}

///Information required for upload command
#[derive(Debug)]
pub struct CloudInfo {
    pub platform: CloudPlatform,
    pub task: CloudTask,
    pub path: String,
    pub config: Config,
}

///Information required for config command
#[derive(Debug)]
pub struct ConfigInfo {
    pub category: String,
    pub value: String,
    pub value2: String,
    pub config: Config,
}

///Passes the directive to the caller
#[derive(Debug)]
pub enum Directive {
    Encrypt(EncryptInfo),
    Decrypt(DecryptInfo),
    Cloud(CloudInfo),
    Config(ConfigInfo),
}

impl Directive {
    ///Processes all directives passed through -- acts as an API
    ///Accepts a directive with the requisite struct and information
    pub fn process_directive(self) {
        match self {
            Directive::Encrypt(info) => Self::process_encrypt(info),
            Directive::Decrypt(info) => Self::process_decrypt(info),
            Directive::Cloud(info) => Self::process_cloud(info),
            Directive::Config(info) => Self::process_config(info),
        }
    }

    ///Process the encryption directive
    pub fn process_encrypt(info: EncryptInfo) {
        //Determine if file or directory
        match PathBuf::from(&info.path).is_dir() {
            //if directory
            true => {
                // get vec of dir
                let dir =
                    walk_directory(&info.path, &info.config).expect("could not find directory!");
                // dbg!(&dir);
                for path in dir {
                    println!("Encrypting file: {}", path.display());
                    encrypt_file(
                        &info.config,
                        path.display().to_string().as_str(),
                        info.in_place.to_owned(),
                    )
                }
            }
            //if file
            false => {
                encrypt_file(&info.config, &info.path, info.in_place);
            }
        };
    }

    ///Process the decryption directive
    fn process_decrypt(info: DecryptInfo) {
        //Determine if file or directory
        match PathBuf::from(&info.path).is_dir() {
            //if directory
            true => {
                // get vec of dir
                let dir =
                    walk_directory(&info.path, &info.config).expect("could not find directory!");
                // dbg!(&dir);
                for path in dir {
                    if path.extension().unwrap() == "crypt" {
                        println!("Decrypting file: {}", path.display());
                        let _ = decrypt_file(
                            &info.config,
                            path.display().to_string().as_str(),
                            info.output.to_owned(),
                        );
                    }
                }
            }
            //if file
            false => {
                let _ = decrypt_file(&info.config, &info.path, info.output.to_owned());
            }
        };
    }

    ///Process the upload information directive
    fn process_cloud(info: CloudInfo) {
        println!("{:#?}", info);
        let runtime = Runtime::new().unwrap();

        //Fetch FileCrypts from crypt_keeper
        let path_info = PathInfo::new(&info.path);
        let paths =
            walk_paths(info.path.as_str(), &info.config).expect("Could not generate path(s)");
        let paths: Vec<PathInfo> = 
            paths.into_iter().filter(|p| p.name != path_info.name).collect();
        println!("{:#?}", paths);

        //Track all folder ids
        let mut folder_ids: HashMap<String, String> = HashMap::new();
        match info.platform {
            CloudPlatform::Google => {
                //Grab user authentication token
                let user_token = oauth::google_access().expect("Could not access user credentials");
                //Access google drive and ensure a crypt folder exists
                let crypt_folder = match runtime.block_on(drive::g_create_folder(
                    &user_token,
                    None,
                    "".to_string(),
                )) {
                    Ok(folder_id) => folder_id,
                    Err(e) => {
                        println!("{}", e);
                        "".to_string()
                    }
                };

                // let _ = runtime.block_on(drive::g_drive_info(&user_token));
                match info.task {
                    CloudTask::Upload => {
                        match path_info.is_dir {
                            true => {
                                //Create the root directory
                                folder_ids.insert(
                                    path_info.full_path.display().to_string(),
                                    runtime.block_on(
                                        drive::g_create_folder(
                                        &user_token,
                                        Some(&PathBuf::from(path_info.name.clone())),
                                        crypt_folder,))
                                    .expect("Could not create directory in google drive") 
                                );
                                //Create all folders relative to the root directory
                                for path in paths.clone() {
                                    let parent_path = path.parent.display().to_string();
                                    println!("{:#?}\n{}", folder_ids, parent_path);
                                    let parent_id = folder_ids.get(&parent_path)
                                        .expect("Could not retrieve parent ID")
                                        .to_string();

                                    if path.is_dir {
                                        folder_ids.insert(
                                            path.full_path.display().to_string(),
                                            runtime.block_on(
                                                drive::g_create_folder(
                                                &user_token,
                                                Some(&PathBuf::from(path.name.clone())),
                                                parent_id))
                                            .expect("Could not create directory in google drive")
                                        );
                                    } 
                                }
                                //Upload every file to their respective parent directory
                                for path in paths {
                                    let parent_path = path.parent.display().to_string();
                                    let parent_id = folder_ids.get(&parent_path)
                                        .expect("Could not retrieve parent ID")
                                        .to_string();

                                    if !path.is_dir {
                                        let _ = runtime.block_on(
                                            drive::g_upload(
                                                user_token.clone(), 
                                                &path.full_path.display().to_string(), 
                                                parent_id)
                                        );
                                    }
                                }
                            }
                            false => {
                                let _ = runtime.block_on(
                                    drive::g_upload(
                                        user_token, &info.path, crypt_folder)
                                );
                            }
                        }
                    }
                    CloudTask::Download => {
                        todo!()
                    }
                    CloudTask::View => {
                        todo!()
                    }
                }
            }
            CloudPlatform::DropBox => {
                match info.task {
                    CloudTask::Upload => {
                        let path = PathBuf::from(info.path.as_str());
                        match path.is_dir() {
                            true => {
                                //If folder, verify that the folder exists, create it otherwise
                            }
                            false => {}
                        }
                        //Determine if it's a file or a folder that's being uploaded
                        todo!()
                    }
                    CloudTask::Download => {
                        todo!()
                    }
                    CloudTask::View => {
                        todo!()
                    }
                }
            }
        }
    }

    ///Processes the configuration change directive TODO: This needs to be redone, something isnt working.
    fn process_config(mut info: ConfigInfo) {
        //Regardles, print the config
        println!("{:#?}", info.config);

        //Process the directive
        match info.category.as_str() {
            "database_path" => match info.value.to_lowercase().as_str() {
                "" => {
                    let path = get_full_file_path(&info.config.database_path)
                        .expect("Error fetching database path");
                    println!("Current Database Path:\n  {}", path.display());
                }
                _ => {
                    println!(
                        "WARNING: changing your database will prevent you from decrypting existing
                     files until you change the path back. ARE YOU SURE? (Y/N)"
                    );

                    let mut s = String::new();
                    while s.to_lowercase() != *"y" || s.to_lowercase() != *"n" {
                        std::io::stdin()
                            .read_line(&mut s)
                            .expect("Did not enter a correct string");
                    }

                    if s.as_str() == "y" {
                        if PathBuf::from(&info.value).exists() {
                            info.config.set_database_path(&info.value2);
                        } else {
                            //TODO: create path
                        }
                        info.config.set_database_path(&info.value2);
                    }
                }
            },

            "retain" => match info.config.set_retain(info.value.to_owned()) {
                true => println!(
                    "Retain changed to: {}",
                    match info.value.as_str() {
                        "true" | "t" => "true",
                        "false" | "f" => "false",
                        _ => unreachable!(),
                    }
                ),
                false => eprintln!("Error occured, please verify parameters."),
            },

            "ignore_directories" => match info.value.to_lowercase().as_str() {
                "add" | "a" => info.config.append_ignore_directories(&info.value2),
                "remove" | "r" => info
                    .config
                    .remove_item_from_ignore_directories(&info.value2),
                _ => println!("invalid input"),
            },

            "zstd_level" => match info.config.set_zstd_level(info.value2.parse().unwrap()) {
                false => println!("Error occured, please verify parameters."),
                true => println!("{} value changed to: {}", info.category, info.value),
            },
            _ => eprintln!(
                "invalid selection!\n use `crypt config` to see available config options."
            ),
        };
    }
}
