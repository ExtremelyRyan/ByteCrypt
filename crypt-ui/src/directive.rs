use crate::cli::{
    KeeperCommand,
    KeeperPurgeSubCommand::{Database, Token},
};
use anyhow::Result;
use crypt_cloud::crypt_core::{
    common::{
        build_tree, chooser, get_crypt_folder, get_full_file_path, send_information,
        walk_crypt_folder, walk_directory, walk_paths, PathInfo,
    },
    config::{self, Config, ConfigTask, ItemsTask},
    db::{
        self, delete_keeper, export_keeper, query_crypt, query_keeper_by_file_name,
        query_keeper_crypt,
    },
    filecrypt::{
        decrypt_contents, decrypt_file, encrypt_file, get_uuid, get_uuid_from_file, FileCrypt,
    },
    filetree::{
        tree::{dir_walk, is_not_hidden, sort_by_name, Directory},
        treeprint::print_tree,
    },
    token::{purge_tokens, UserToken},
};
use crypt_cloud::drive;
use std::{collections::HashMap, path::PathBuf};
use tokio::runtime::Runtime;

#[derive(Debug)]
enum CloudError {
    ///Error accessing Crypt "root" folder
    CryptFolderError,
    RuntimeError,
}

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
pub fn encrypt(path: &str, _in_place: bool, output: Option<String>) {
    let buf = PathBuf::from(path);
    //Determine if file or directory
    if buf.is_dir() {
        // get vec of dir
        let dir = walk_directory(path);

        match dir {
            Ok(d) => {
                for p in d {
                    send_information(vec![format!("Encrypting file: {}", p.display())]);
                    encrypt_file(&p.display().to_string(), &output)
                }
            }
            Err(_) => todo!(),
        }
    } else if buf.is_file() {
        encrypt_file(path, &output);
    }
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
                    let res = decrypt_file(path.display().to_string().as_str(), output.to_owned());
                    println!("{res:?}");
                }
            }
        }
        //if file
        false => {
            let res = decrypt_file(path, output.to_owned());
            println!("{res:?}");
        }
    };
}

fn google_startup() -> Result<(Runtime, UserToken, String), CloudError> {
    let runtime = match Runtime::new() {
        core::result::Result::Ok(it) => it,
        Err(_err) => return Err(CloudError::RuntimeError),
    };

    let user_token = UserToken::new_google();

    //Access google drive and ensure a crypt folder exists, create if doesn't
    let crypt_folder: String = match runtime.block_on(drive::g_create_folder(&user_token, None, ""))
    {
        core::result::Result::Ok(folder_id) => folder_id,
        Err(error) => {
            send_information(vec![format!("{}", error)]);
            return Err(CloudError::CryptFolderError);
        }
    };

    Ok((runtime, user_token, crypt_folder))
}

pub fn google_upload2(path: &str) -> Result<()> {
    // let crypt_root = get_crypt_folder();
    let dir = walk_crypt_folder().unwrap_or_default();

    // if there are no files in the crypt folder, return
    if dir.is_empty() {
        return Ok(());
    }
    let res = chooser(dir, path);

    // user aborted
    if res.to_string_lossy() == "" {
        return Ok(());
    }

    dbg!("{}", res.display());

    // determine if path picked is a file or path
    match res.is_file() {
        true => {
            let (runtime, user_token, crypt_folder) = match google_startup() {
                Ok(res) => res,
                Err(_) => todo!(), // TODO: do we handle this here? or do we pass back to CLI?
            };

            // 1. get crypt info from pathbuf
            let mut fc = match get_uuid_from_file(res.clone()) {
                Ok(uuid) => db::query_crypt(uuid)?,
                Err(err) => panic!("{}", err),
            };

            // 2. upload file to cloud, saving drive id to crypt
            fc.drive_id = runtime.block_on(drive::g_upload(
                &user_token,
                &res.display().to_string(),
                &crypt_folder,
                &false,
            ))?;

            // 3. update database.
            db::insert_crypt(&fc)?;

            // 4. show cloud directory
            let cloud_directory = runtime
                .block_on(drive::g_walk(&user_token, "Crypt"))
                .expect("Could not view directory information");
            send_information(build_tree(&cloud_directory));
        }
        false => todo!(),
    }

    Ok(())
}

pub fn google_upload(path: &str, no_encrypt: &bool) {
    let (runtime, user_token, crypt_folder) = match google_startup() {
        Ok(res) => res,
        Err(_) => todo!(), // TODO: do we handle this here? or do we pass back to CLI?
    };

    //Track all folder ids
    let mut folder_ids: HashMap<String, String> = HashMap::new();

    //Get walk path given and build a list of PathInfos
    let path_info = PathInfo::new(path);
    let paths: Vec<PathInfo> = walk_paths(path)
        .into_iter()
        .filter(|p| p.is_dir || p.name.contains(".crypt"))
        .collect();
    let (mut folders, files): (Vec<_>, Vec<_>) = paths.into_iter().partition(|p| p.is_dir);
    folders.sort_by(|a, b| a.full_path.cmp(&b.full_path));

    //Create a hashmap relating PathInfo to FileCrypt for relevant .crypt files
    let mut crypts: HashMap<PathInfo, FileCrypt> = HashMap::new();
    for file in files.clone().iter() {
        let contents = &std::fs::read(file.full_path.display().to_string().as_str()).unwrap();
        let (uuid, _) = get_uuid(contents).expect("Could not retrieve UUID from the file");
        let filecrypt = query_crypt(uuid).expect("Could not query keeper");
        crypts.insert(file.to_owned(), filecrypt);
    }

    //Remove the root directory
    let folders: Vec<PathInfo> = folders
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
                        Some(&PathBuf::from(&path_info.name)),
                        &crypt_folder,
                    ))
                    .expect("Could not create directory in google drive"),
            );
            //Create all folders relative to the root directory
            for folder in folders {
                let parent_path = folder.parent.display().to_string();
                let parent_id = folder_ids
                    .get(&parent_path)
                    .expect("Could not retrieve parent ID")
                    .to_string();

                folder_ids.insert(
                    folder.full_path.display().to_string(),
                    runtime
                        .block_on(drive::g_create_folder(
                            &user_token,
                            Some(&PathBuf::from(&folder.name)),
                            &parent_id,
                        ))
                        .expect("Could not create directory in google drive"),
                );
            }
            //Upload every file to their respective parent directory
            for file in files {
                //Get the parent folder path
                let parent_path = file.parent.display().to_string();
                let parent_id = folder_ids
                    .get(&parent_path)
                    .expect("Could not retrieve parent ID")
                    .to_string();

                //Determine if the file already exists in the google drive
                let drive_id = &crypts.get(&file).unwrap().drive_id;
                let exists = if !drive_id.is_empty() {
                    runtime
                        .block_on(drive::g_id_exists(&user_token, drive_id))
                        .expect("Could not query Google Drive")
                } else {
                    false
                };

                //Only if the file doesn't exist should it be uploaded
                if !exists {
                    let file_id = runtime.block_on(drive::g_upload(
                        &user_token,
                        &file.full_path.display().to_string(),
                        &parent_id,
                        no_encrypt,
                    ));
                    //Update the FileCrypt's drive_id
                    crypts
                        .entry(file.clone())
                        .and_modify(|fc| fc.drive_id = file_id.unwrap());
                } else {
                    let _ = runtime.block_on(drive::g_update(
                        &user_token,
                        drive_id,
                        &file.full_path.display().to_string(),
                    ));
                }
            }
        }
        //Individual file(s)
        false => {
            let file_id = runtime.block_on(drive::g_upload(
                &user_token,
                &path_info.full_path.display().to_string(),
                &crypt_folder,
                no_encrypt,
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
        println!("{:?}", value);
    }

    // TESTING PORPISES
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

pub fn google_download(path: &String) {
    let (runtime, user_token, _) = match google_startup() {
        Ok(res) => res,
        Err(_) => todo!(), // TODO: do we handle this here? or do we pass back to CLI?
    };

    // TODO: how do we handle paths that do not match / misspelled / mis-cased?
    // TODO:

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

    let bytes = runtime
        .block_on(drive::google_query_file(&user_token, &fc.drive_id))
        .unwrap_or(vec![]);

    // TODO: if something went wrong, what do?
    if bytes.is_empty() {
        send_information(vec![format!(
            "Failed to get contents of cloud file. Please try again."
        )]);
        std::process::exit(2);
    }

    // Step 2.5: unzip / decrypt contents / write to file.
    _ = decrypt_contents(fc, bytes);

    // let res = runtime.block_on(g_walk(&user_token, "Crypt")).unwrap();
    // println!("{res:#?}");
}

pub fn google_view(path: &str) {
    let (runtime, user_token, _) = match google_startup() {
        Ok(res) => res,
        Err(_) => todo!(), // TODO: do we handle this here? or do we pass back to CLI?
    };

    let cloud_directory = runtime
        .block_on(drive::g_walk(&user_token, path))
        .expect("Could not view directory information");
    send_information(build_tree(&cloud_directory));
}

pub fn dropbox_upload(_path: &str) {}
pub fn dropbox_download(_path: &str) {}
pub fn dropbox_view(_path: &str) {}

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
                }
            }
        },

        ConfigTask::CryptPath => {
            match path.to_lowercase().as_str() {
                "" => {
                    let path = get_full_file_path(&config.crypt_path);
                    send_information(vec![format!("Current crypt Path:\n  {}", path.display())]);
                }
                _ => {
                    send_information(vec![format!(
                        "{} {}",
                        "WARNING: changing your crypt file path will desync existing crypt files in the cloud",
                        "until you change the path back. ARE YOU SURE? (Y/N)"
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
                            config.set_crypt_path(path);
                        } else {
                            //TODO: create path
                        }
                    }
                }
            };
        }

        ConfigTask::IgnoreItems(option, item) => match option {
            ItemsTask::Add => config.append_ignore_items(&item),
            ItemsTask::Remove => config.remove_ignore_item(&item),
            ItemsTask::Default => {
                let default = Config::default();
                config.set_ignore_items(default.ignore_items);
            }
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
        ConfigTask::IgnoreHidden(_) => todo!(),
        ConfigTask::Hwid => {
            if path.is_empty() {
                send_information(vec![format!("{}", config.get_system_name())]);
            } else {
                send_information(vec![format!("changing system name to: {}", path)]);
            }
            config.set_system_name(path);
        }
    };
}

pub fn keeper(kc: &KeeperCommand) {
    match kc {
        KeeperCommand::Import { path } => {
            KeeperCommand::import(path);
        }
        KeeperCommand::Export { alt_path } => {
            // TODO: Fix this?
            if alt_path.is_empty() {
                _ = export_keeper(None);
            } else {
                _ = export_keeper(Some(alt_path));
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
        //List
        KeeperCommand::List {} => {
            let fc = query_keeper_crypt().unwrap();
            for crypt in fc {
                println!(
                    "file: {}{} \nfull file path: {}\ncloud location: {}\n",
                    crypt.filename,
                    crypt.ext,
                    crypt.full_path.display(),
                    crypt.drive_id,
                );
            }
        }
    }
}

pub fn test() {}

pub fn ls(local: &bool, cloud: &bool) {
    let crypt_root = get_crypt_folder();

    let dir: Directory = dir_walk(&crypt_root.clone(), is_not_hidden, sort_by_name).unwrap();

    match (local, cloud) {
        // display both
        (true, true) => todo!(),
        // display local only
        (_, false) => print_tree(crypt_root.to_str().unwrap(), &dir),
        // display cloud only
        (_, true) => google_view("Crypt"),
    };
}
