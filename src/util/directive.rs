use crate::{
    database::crypt_keeper,
    util::{
        config::Config,
        encryption::{decrypt_file, encrypt_file},
        parse::write_contents_to_file,
        path::{get_full_file_path, walk_directory},
    },
};
use std::{ffi::OsStr, path::PathBuf};

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

///Information required for upload command
#[derive(Debug)]
pub struct UploadInfo {
    pub placeholder: bool,
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
    Upload(UploadInfo),
    Config(ConfigInfo),
}

impl Directive {
    ///Processes all directives passed through -- acts as an API
    ///Accepts a directive with the requisite struct and information
    pub fn process_directive(self) {
        match self {
            Directive::Encrypt(info) => Self::process_encrypt(info),
            Directive::Decrypt(info) => Self::process_decrypt(info),
            Directive::Upload(info) => Self::process_upload(info),
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
    fn process_upload(_info: UploadInfo) {
        todo!();
    }

    ///Processes the configuration change directive TODO: This needs to be redone, something isnt working.
    fn process_config(mut info: ConfigInfo) {
        match info.category.as_str() {
            "database_path" => match info.value.to_lowercase().as_str() {
                "" => {
                    let path = get_full_file_path(&info.config.database_path)
                        .expect("Error fetching database path");
                    println!("Current Database Path:\n  {}", 
                        path.display());
                },
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
                            // create path
                        }
                        info.config.set_database_path(&info.value2);
                    }
                }
            },

            // "cloud_services" => todo!(),
            "retain" => match info.config.set_retain(info.value.to_owned()) {
                false => eprintln!("Error occured, please verify parameters."),
                true => println!("{} value changed to: {}", info.category, info.value),
            },

            "hidden_directories" => match info.value.to_lowercase().as_str() {
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
