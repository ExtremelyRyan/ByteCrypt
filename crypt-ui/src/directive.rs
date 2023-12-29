use crate::cli::{
    KeeperCommand,
    KeeperPurgeSubCommand::{Database, Token},
};
use crypt_cloud::crypt_core::{
    common::{
        build_tree, get_full_file_path, send_information, walk_directory, walk_paths, PathInfo,
    },
    config::{self, Config, ConfigTask, ItemsTask},
    db::{self, query_crypt},
    db::{delete_keeper, export_keeper, query_keeper_crypt, query_keeper_by_file_name},
    filecrypt::{decrypt_file, encrypt_file, get_uuid, FileCrypt, decrypt_contents},
    token::{purge_tokens, CloudTask},
    token::{CloudService, UserToken},
};
use crypt_cloud::drive::{self, g_walk};
use std::{collections::HashMap, path::PathBuf};
use tokio::runtime::Runtime;

///Process the encryption directive
///
/// # Example
///```ignore
/// # use crypt_lib::util::directive::Directive;
/// let in_place = false;
/// let output = "desired/output/path".to_string();
///
/// let directive = Directive::new("relevant/file.path".to_string());
/// directive.encrypt(in_place, output);
///```
///TODO: implement output
pub fn encrypt(path: &str, in_place: bool, _output: Option<String>) {
    //Determine if file or directory
    match PathBuf::from(path).is_dir() {
        //if directory
        true => {
            // get vec of dir
            let dir = walk_directory(path).expect("could not find directory!");
            // dbg!(&dir);
            for path in dir {
                send_information(vec![format!("Encrypting file: {}", path.display())]);
                encrypt_file(path.display().to_string().as_str(), in_place)
            }
        }
        //if file
        false => {
            encrypt_file(path, in_place);
        }
    };
}

///Process the decryption directive
///
/// # Example
///```ignore
/// # use crypt_lib::util::directive::Directive;
/// let in_place = false;
/// let output = "desired/output/path".to_string();
///
/// let directive = Directive::new("relevant/file.path".to_string());
/// directive.decrypt(in_place, output);
///```
///TODO: rework output for in-place
///TODO: implement output to just change save_location
pub fn decrypt(path: &str, _in_place: bool, output: Option<String>) {
    //Determine if file or directory
    match PathBuf::from(path).is_dir() {
        //if directory
        true => {
            // get vec of dir
            let dir = walk_directory(path).expect("could not find directory!");
            // dbg!(&dir);
            for path in dir {
                if path.extension().unwrap() == "crypt" {
                    send_information(vec![format!("Decrypting file: {}", path.display())]);
                    let _ = decrypt_file(path.display().to_string().as_str(), output.to_owned());
                }
            }
        }
        //if file
        false => {
            let _ = decrypt_file(path, output.to_owned());
        }
    };
}

///View, upload, or download files from supported cloud service
///
/// # Example
///```ignore
/// # use crypt_lib::util::directive::Directive;
/// let platform = CloudPlatform::Google;
/// let task = CloudTask::Upload;
///
/// let directive = Directive::new("relevant/file.path".to_string());
/// directive.cloud(platform, task);
///```
pub fn cloud(path: &str, platform: CloudService, task: CloudTask) {
    //For async functions
    let runtime = Runtime::new().unwrap();

    //Actions depend on the platform
    match platform {
        //Google
        CloudService::Google => {
            //Grab user authentication token
            let user_token = UserToken::new_google();

            //Access google drive and ensure a crypt folder exists, create if doesn't
            let crypt_folder = match runtime.block_on(drive::g_create_folder(&user_token, None, ""))
            {
                Ok(folder_id) => folder_id,
                Err(error) => {
                    send_information(vec![format!("{}", error)]);
                    "".to_string()
                }
            };
            // let _ = runtime.block_on(drive::g_drive_info(&user_token));
            match task {
                CloudTask::Upload => {
                    //Track all folder ids
                    let mut folder_ids: HashMap<String, String> = HashMap::new();
                    //Get walk path given and build a list of PathInfos
                    let path_info = PathInfo::new(path);
                    let paths = walk_paths(path);
                    //Create a hashmap relating PathInfo to FileCrypt for relevant .crypt files
                    let mut crypts: HashMap<PathInfo, FileCrypt> = HashMap::new();
                    for file in paths.clone().iter() {
                        if !file.is_dir && file.name.contains(".crypt") {
                            let contents =
                                &std::fs::read(file.full_path.display().to_string().as_str())
                                    .unwrap();
                            let (s, _) = get_uuid(contents);
                            let fc = query_crypt(s).expect("Could not query keeper");
                            crypts.insert(file.to_owned(), fc);
                        }
                    }
                    //Remove the root directory
                    let paths: Vec<PathInfo> = paths
                        .into_iter()
                        .filter(|p| p.name != path_info.name)
                        .collect();

                    //Match if directory or file
                    match path_info.is_dir {
                        // Full directory upload
                        true => {
                            //Create the root directory
                            folder_ids.insert(
                                path_info.full_path.display().to_string(),
                                runtime
                                    .block_on(drive::g_create_folder(
                                        &user_token,
                                        Some(&PathBuf::from(path_info.name.clone())),
                                        &crypt_folder,
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
                                                &parent_id,
                                            ))
                                            .expect("Could not create directory in google drive"),
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
                                if path.name.contains(".crypt") {
                                    let drive_id = crypts.get(&path).unwrap().drive_id.as_str();
                                    if !drive_id.is_empty() {
                                        let exists = runtime
                                            .block_on(drive::g_id_exists(&user_token, drive_id));

                                        println!("{:?}", exists);
                                    }
                                }

                                if !path.is_dir {
                                    let file_id = runtime.block_on(drive::g_upload(
                                        &user_token,
                                        &path.full_path.display().to_string(),
                                        &parent_id,
                                    ));
                                    //Update the FileCrypt's drive_id
                                    if path.name.contains(".crypt") {
                                        crypts
                                            .entry(path)
                                            .and_modify(|fc| fc.drive_id = file_id.unwrap());
                                    }
                                }
                            }
                        }
                        //Individual file(s)
                        false => {
                            let file_id = runtime.block_on(drive::g_upload(
                                &user_token,
                                &path_info.full_path.display().to_string(),
                                &crypt_folder,
                            ));
                            //Update the FileCrypt's drive_id
                            if path_info.name.contains(".crypt") {
                                crypts
                                    .entry(path_info)
                                    .and_modify(|fc| fc.drive_id = file_id.unwrap());
                            }
                        }
                    }
                    //Update the keeper with any changes to FileCrypts
                    for (_, value) in crypts {
                        let _ = db::insert_crypt(&value);
                    }

                    //TESTING PORPISES
                    let after_upload_keeper = db::query_keeper_crypt().unwrap();
                    for item in after_upload_keeper {
                        println!("file: {}{}", item.filename, item.ext);
                        println!("full path: {}", item.full_path.display());
                        println!("drive ID: {}", item.drive_id);
                    }
                    //Print the directory
                    let cloud_directory = runtime
                        .block_on(drive::g_walk(&user_token, "Crypt"))
                        .expect("Could not view directory information");
                    send_information(build_tree(&cloud_directory));
                }
                CloudTask::Download => {

                    // OK, so what are we wanting to do here? we are looking at the list of files on the cloud
                    // from running `crypt cloud -g view`

                    // `path` that we are getting from the user is the filename (should not have ext, but might)
                    // so we can query for that from the db.

                    // Step 1: get path from the user and verify it exists in our database.
                    println!("path {}", path);
                    let fc = query_keeper_by_file_name(path).unwrap();

                    // TODO: Step 1.1: if multiple filecrypts exist for the same filename, then perhaps it's just easier
                    // if we download the file, and check uuid.
                    // thought about having user select, but based off what? filename, the "fullpath" we have in the db?

                    
                    // step 2: get drive id and query file, retreve contents

                    let bytes = runtime.block_on(drive::google_query_file(
                        &fc.drive_id,
                        user_token.clone(),
                    )).unwrap_or(vec![]);

                    // Step 2.5: unzip / decrypt contents / write to file.
                    _ = decrypt_contents(fc, bytes);


                    // let res = runtime.block_on(g_walk(&user_token, "Crypt")).unwrap();
                    // println!("{res:#?}");
                }
                CloudTask::View => {
                    let cloud_directory = runtime
                        .block_on(drive::g_walk(&user_token, path))
                        .expect("Could not view directory information");
                    send_information(build_tree(&cloud_directory));
                }
            }
        }
        CloudService::Dropbox => {
            match task {
                CloudTask::Upload => {
                    let path = PathBuf::from(path);
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

///Change configuration settings
///
/// # Example
///```ignore
/// # use crypt_lib::util::directive::Directive;
/// let add_remove = ItemTask::Add;
/// let item = "ignore.txt".to_string();
///
/// let directive = Directive::new("relevant/file.path".to_string());
/// directive.config(add_remove, item);
///```
pub fn config(path: &str, config_task: ConfigTask) {
    let mut config = config::get_config_write();

    //Process the directive
    match config_task {
        ConfigTask::DatabasePath => match path.to_lowercase().as_str() {
            "" => {
                let path = get_full_file_path(&config.database_path);
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
                    if PathBuf::from(path).exists() {
                        config.set_database_path(path);
                    } else {
                        //TODO: create path
                    }
                    config.set_database_path(path);
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

pub fn keeper(kc: &KeeperCommand) {
    match kc {
        KeeperCommand::Import { path } => {
            KeeperCommand::import(path);
        }
        KeeperCommand::Export { alt_path } => {
            let res;
            if alt_path.is_empty() {
                res = export_keeper(None);
            } else {
                res = export_keeper(Some(&alt_path))
            }
            match res {
                Ok(_) => (),
                Err(e) => panic!("problem exporting keeper! {}", e),
            };
        }
        KeeperCommand::Purge { category } => match category {
            Some(Token {}) => purge_tokens(),
            Some(Database {}) => {
                send_information(vec![
                    format!("==================== WARNING ===================="),
                    format!("DOING THIS WILL IRREVERSIBLY DELETE YOUR DATABASE\n"),
                    format!("DOING THIS WILL IRREVERSIBLY DELETE YOUR DATABASE\n"),
                    format!("DOING THIS WILL IRREVERSIBLY DELETE YOUR DATABASE\n\n"),
                    format!(r#"type "delete database" to delete, or "q" to quit"#),
                ]);
                let mut phrase = String::new();
                let match_phrase = String::from("delete database");
                loop {
                    std::io::stdin()
                        .read_line(&mut phrase)
                        .expect("Failed to read line");
                    phrase = phrase.trim().to_string();

                    if phrase.eq(&match_phrase) {
                        break;
                    }
                    if phrase.eq("q") {
                        return;
                    }
                }
                _ = delete_keeper();
                send_information(vec![format!("database was deleted.")]);
            }
            None => send_information(vec![format!("invalid entry entered.")]),
        },
        //
        KeeperCommand::List {} => {
            let fc = query_keeper_crypt().unwrap();
            for crypt in fc {
                println!(
                    "{}",
                    format!(
                        "file: {}{} \nfull file path: {}\ncloud location: {}\n",
                        crypt.filename,
                        crypt.ext,
                        crypt.full_path.display(),
                        crypt.drive_id,
                    )
                );
            }
        }
    }
}
// }
