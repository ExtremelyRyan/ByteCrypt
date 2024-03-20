use crate::{
    cli::{
        KeeperCommand,
        KeeperPurgeSubCommand::{Database, Token},
    },
    error,
    prelude::*,
};
use crypt_cloud::{
    crypt_core::{
        common::{
            build_tree, chooser, get_crypt_folder, get_filenames_from_subdirectories,
            get_full_file_path, send_information, verify_path, walk_crypt_folder, walk_directory,
        },
        config::{self, Config, ConfigTask, ItemsTask},
        db::{self, delete_keeper, export_keeper, query_crypt, query_keeper_crypt},
        filecrypt::{decrypt_contents, decrypt_file, encrypt_file, get_uuid_from_file},
        filetree::{
            tree::{dir_walk, is_not_hidden, sort_by_name, Directory},
            treeprint::print_tree,
        },
        token::{purge_tokens, UserToken},
    },
    drive,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio::runtime::Runtime;
// #############################################################################################################################################

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
pub fn encrypt(path: &str, output: Option<String>) -> Result<()> {
    // verify our path is pointing to a actual dir/file
    if !verify_path(&path) {
        send_information(vec![format!("could not find path: {}", path)]);
        return Ok(());
    }

    // get the difference between the user's current working directory, and the path they passed in.

    let mut root = PathBuf::new();
    let user_path: PathBuf = PathBuf::from(path);

    //Determine if file or directory
    match user_path.is_dir() {
        true => {
            if let Ok(directory) = walk_directory(path, false) {
                for path in directory {
                    if path.is_dir() {
                        root.push(path.file_name().unwrap());
                    } else if path.is_file() {
                        encrypt_file(path.to_str().unwrap(), &Some(root.display().to_string()));
                    }
                }
            }
        }
        false => encrypt_file(path, &output),
    }
    Ok(())
}

///Process the decryption directive
pub fn decrypt(path: &str, output: Option<String>) {
    let mut crypt_folder = get_crypt_folder();
    crypt_folder.push(path);

    let mut root = PathBuf::new();
    println!(
        "PathBuf::from(path).is_dir(): {}",
        PathBuf::from(path).is_dir()
    );
    //Determine if file or directory
    match PathBuf::from(path).is_dir() {
        //directory
        true => {
            // get vec of dir
            if let Ok(directory) = walk_directory(crypt_folder, false) {
                for p in directory {
                    if p.is_dir() {
                        root.push(p.file_name().unwrap());
                    } else if p.is_file() {
                        send_information(vec![format!("Decrypting file: {}", p.display())]);
                        let _res = decrypt_file(p, root.display().to_string());
                    }
                }
            }
        }
        // file
        false => {
            let res;
            if let Some(o) = output {
                res = decrypt_file(path, o);
            } else {
                res = decrypt_file(path, "".to_string());
            }
            println!("decrypt result: {:?}", res);
        }
    };
}

// ############################################ Cloud Implementation ############################################

/// Contains the necessary properties for Google Drive
pub struct Google {
    pub runtime: Runtime,
    pub token: UserToken,
    pub cloud_root_folder: String,
}

impl Google {
    /// Creates a new [`Google`].
    fn new() -> Result<Self> {
        let runtime = Runtime::new()?;

        let token = UserToken::new_google();

        // Access google drive and ensure a crypt folder exists, create if doesn't
        let cloud_root_folder: String =
            runtime.block_on(drive::g_create_folder(&token, None, ""))?;

        return Ok(Self {
            runtime,
            token,
            cloud_root_folder,
        });
    }
}

// ############################################ Cloud Upload ############################################

pub fn google_upload() -> Result<()> {
    let user_result = chooser("").unwrap_or_default();

    // user aborted | no files in crypt
    if user_result.to_string_lossy().is_empty() {
        return Err(Error::UploadError(error::UploadError::UserAbortedError));
    }

    let google = Google::new()?;

    // determine if path picked is a file or path
    if user_result.is_file() {
        // 1. get crypt info from pathbuf
        let mut fc = get_uuid_from_file(user_result.clone()).and_then(db::query_crypt)?;

        // 2. upload file to cloud, saving drive id to crypt
        fc.drive_id = google.runtime.block_on(drive::g_upload(
            &google.token,
            &user_result.display().to_string(),
            &google.cloud_root_folder,
        ))?;

        // 3. update database.
        db::insert_crypt(&fc)?;

        // 4. show cloud directory
        let cloud_directory = google
            .runtime
            .block_on(drive::g_walk(&google.token, "Crypt"))
            .expect("Could not view directory information");
        send_information(build_tree(&cloud_directory));
    } else {
        // get all our file paths from folder
        let (files, _) = walk_crypt_folder()?;

        for file in files {
            // get FileCrypt information from keeper
            let mut fc = match get_uuid_from_file(file.as_path()) {
                Ok(uuid) => db::query_crypt(uuid).unwrap(),
                Err(_) => continue,
            };

            // check if we have a drive id in the filecrypt & if it exists in google drive
            // if so: we will update the file instead of overwriting it.
            // if we fail, blank out the FC drive id and fall through to upload.
            if !fc.drive_id.is_empty()
                && google
                    .runtime
                    .block_on(drive::g_id_exists(&google.token, &fc.drive_id))
                    .is_ok_and(|x| x)
            {
                fc.drive_id = google
                    .runtime
                    .block_on(drive::g_update(
                        &google.token,
                        &fc.drive_id,
                        &file.to_string_lossy(),
                    ))
                    .unwrap_or_else(|_| "".to_string());

                if !fc.drive_id.is_empty() {
                    continue;
                }
            }

            // Find the position of "crypt" in the path
            if let Some(index) = file.iter().position(|component| component == "crypt") {
                // Collect the components after "crypt"
                let remaining_components: Vec<_> = file.iter().skip(index + 1).collect();

                // Check if there are remaining components
                if remaining_components.is_empty() {
                    continue;
                }

                // our parent directory ID
                let mut parent: String = google.cloud_root_folder.clone();

                // our current directory ID
                let mut current: String = String::new();

                // length of remaining components
                let len = remaining_components.len() - 1;

                // Iterate over each remaining component
                for (num, component) in remaining_components.iter().enumerate() {
                    if num != len {
                        current = google.runtime.block_on(drive::g_create_folder(
                            &google.token,
                            Some(&PathBuf::from(component)),
                            &parent,
                        ))?;
                        parent = current.clone();
                    } else {
                        current = google.runtime.block_on(drive::g_upload(
                            &google.token,
                            file.to_str().unwrap(),
                            &current,
                        ))?;
                        fc.drive_id = current.clone();
                    }
                }
                // 3. update database.
                db::insert_crypt(&fc)?;
            }
        }
    }

    Ok(())
}

// ############################################ Cloud Download ############################################

pub fn google_download(path: &str) -> Result<()> {
    let google = Google::new()?;

    let crypt_folder = get_crypt_folder();
    let (_files, _) = get_filenames_from_subdirectories(crypt_folder)?;

    let file_choice = chooser(path)?;
    dbg!(&file_choice);

    if file_choice.is_file() {
        // get uuid from file
        let uuid = get_uuid_from_file(&file_choice)?;

        // Step 1: get path from the user and verify it exists in our database.
        let fc = query_crypt(uuid)?;
        dbg!(&fc);

        // TODO: Step 1.1: if multiple filecrypts exist for the same filename, then perhaps it's just easier
        // if we download the file, and check uuid.
        // thought about having user select, but based off what? filename, the "fullpath" we have in the db?

        // step 2: get drive id and query file, retreve contents

        let bytes = google
            .runtime
            .block_on(drive::google_query_file(&google.token, &fc.drive_id))
            .unwrap_or(vec![]);

        // TODO: if something went wrong, what do?
        if bytes.is_empty() {
            send_information(vec![format!(
                "Failed to get contents of cloud file. Please try again."
            )]);
            std::process::exit(2);
        }

        // Step 2.5: unzip / decrypt contents / write to file.
        decrypt_contents(fc, bytes)?;
    }
    // otherwise we assume it is a folder
    else {
        // query google drive for folder and get all files

        // iterate through files and get drive id's

        // download each file.
    }

    Ok(())
}

// ############################################ Cloud View ############################################

pub fn google_view(path: &str) -> Result<()> {
    let google = Google::new()?;

    let cloud_directory = google
        .runtime
        .block_on(drive::g_walk(&google.token, path))
        .expect("Could not view directory information");
    send_information(build_tree(&cloud_directory));

    return Ok(());
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

// Function to write the file to the base file path
pub fn merge_base_with_relative_path(
    base_path: &Path,
    relative_file_path: &Path,
) -> Result<PathBuf> {
    // Extract the folder structure relative to the current working directory
    let relative_path = relative_file_path
        // Assuming the fike path is relative to the current working directory
        .strip_prefix(Path::new("."))
        .unwrap_or(relative_file_path);

    // Create the target path by joining the base path and the relative path
    let mut target_path = base_path.join(relative_path);

    // Create directories if they don't exist
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Check if the target file already exists
    let mut counter = 1;
    while target_path.exists() {
        // Modify the filename by appending a counter
        let filename_with_counter = format!(
            "{} ({})",
            target_path.file_stem().unwrap().to_string_lossy(),
            counter
        );

        // Create the new target path with the modified filename
        target_path = target_path.with_file_name(filename_with_counter);
        counter += 1;
    }

    Ok(target_path)
}

pub fn ls(local: &bool, cloud: &bool) {
    let crypt_root = get_crypt_folder();

    let dir: Directory = dir_walk(&crypt_root.clone(), is_not_hidden, sort_by_name).unwrap();

    match (local, cloud) {
        // display both
        (true, true) => todo!(),
        // display local only
        (_, false) => print_tree(crypt_root.to_str().unwrap(), &dir),
        // display cloud only
        (_, true) => _ = google_view("Crypt"),
    };
}

// ===========================================================
// DANGER ZONE ===============================================
// ===========================================================

pub fn test() {
    // let (runtime, user_token, crypt_folder) = match google_startup() {
    // Ok(res) => res,
    // Err(_) => todo!(), // TODO: do we handle this here? or do we pass back to CLI?
    // };
    // let res = runtime.block_on(drive::g_view(&user_token, "Crypt"));
    // println!("{:#?}", res);

    // let res = walk_directory("test_folder", false);
    // println!("{:#?}", res);

    // let res = runtime.block_on(drive::google_query_folders(&user_token, "testerer_folderer",&crypt_folder));
    // println!("{:#?}", res);

    // let res = runtime.block_on(drive::google_query(&user_token, &crypt_folder));
    // println!("{:#?}", res);
    let crypt = get_crypt_folder();
    let (mut left, mut right) = get_filenames_from_subdirectories(crypt).unwrap();
    left.append(&mut right);

    let res = chooser("");
    println!("{:#?}", res);
}
