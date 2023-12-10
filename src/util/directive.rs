use std::path::PathBuf; 
use clap::builder::OsStr;
use crate::{
    database::crypt_keeper,
    util::{
        config::Config,
        encryption::{decrypt_file, encrypt_file},
        parse::write_contents_to_file,
        path::{get_full_file_path, walk_directory},
    },
};


///Passes the directive to the caller
#[derive(Debug)]
pub enum Directive {
    Encrypt(EncryptInfo),
    Decrypt(DecryptInfo),
    Upload(UploadInfo),
    Config(ConfigInfo),
}

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
    pub update: String,
    pub value: String,
    pub value2: String,
    pub config: Config,
}

///Processes all directives passed through -- acts as an API
///Accepts a directive with the requisite struct and information
pub fn process_directive(directive: Directive) -> anyhow::Result<()> {
    match directive {
        Directive::Encrypt(info) => { process_encrypt(info)? },
        Directive::Decrypt(info) => { process_decrypt(info)? },
        Directive::Upload(info) => { process_upload(info)? },
        Directive::Config(info) => { process_config(info)? },
    }
    return Ok(());
}

///Process the encryption directive
fn process_encrypt(info: EncryptInfo) -> anyhow::Result<()> {
    //Determine if file or directory
    match PathBuf::from(&info.path).is_dir() {
        //if directory
        true => {
            // get vec of dir
            let dir = walk_directory(&info.path, &info.config).expect("could not find directory!");
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
    return Ok(());
}

///Process the decryption directive
fn process_decrypt(info: DecryptInfo) -> anyhow::Result<()> {
    //Determine if file or directory
    match PathBuf::from(&info.path).is_dir() {
        //if directory
        true => {
            // get vec of dir
            let dir = walk_directory(&info.path, &info.config).expect("could not find directory!");
            // dbg!(&dir);
            for path in dir {
                if path.extension() == Some(&OsStr::from("crypt")) {
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
    return Ok(());
}

///Process the upload information directive
fn process_upload(info: UploadInfo) -> anyhow::Result<()> {
    info.placeholder; //just to get rid of warnings TODO: remove
    todo!();

    // return Ok(());
}

///Processes the configuration change directive
fn process_config(mut info: ConfigInfo) -> anyhow::Result<()> {
    if info.value.is_empty() {
        println!("cannot update {}, missing update value", info.update);
        return Ok(()); // TODO: fix this later
    }
    match info.update.as_str() {
        // TODO set path
        "database_path" => match info.value.to_lowercase().as_str() {
            "get" | "g" => println!(
                "database_path: {}",
                get_full_file_path(info.config.get_database_path())?.display()
            ),
            "set" | "s" => {
                println!("WARNING: changing your database will prevent you from decrypting existing
                     files until you change the path back. ARE YOU SURE? (Y/N)");

                let mut s = String::new();
                while s.to_lowercase() != String::from("y")
                    || s.to_lowercase() != String::from("n")
                {
                    std::io::stdin()
                        .read_line(&mut s)
                        .expect("Did not enter a correct string");
                }

                if s.as_str() == "y" {
                    if PathBuf::from(&info.value2).exists() {
                        info.config.set_database_path(&info.value2);
                    } else {
                        // create path
                    }
                    info.config.set_database_path(&info.value2);
                }
            }
            _ => println!("not valid"),
        },
        // TODO: add / remove items in list
        // "cloud_services" => todo!(),
        "retain" => match info.config.set_retain(info.value.to_owned()) {
            false => eprintln!("Error occured, please verify parameters."),
            true => println!("{} value changed to: {}", info.update, info.value),
        },
        "hidden_directories" => match info.value.to_lowercase().as_str() {
            "add" | "a" => info.config.append_ignore_directories(&info.value2),
            "remove" | "r" => info.config.remove_item_from_ignore_directories(&info.value2),
            _ => println!("invalid input"),
        },
        _ => eprintln!(
            "invalid selection!\n use -s | --show to see available config options."
        ),
    }
    return Ok(());
}
