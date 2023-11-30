use crate::{
    database::crypt_keeper,
    util::{self, *, parse::write_contents_to_file},
};
use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

use super::tui;

///Passes the directive to the caller
#[derive(Debug)]
pub enum Directive {
    Encrypt(EncryptInfo),
    Decrypt(DecryptInfo),
}

///Information required for an encryption command
#[derive(Debug)]
pub struct EncryptInfo {
    is_directory: bool,
    path: Vec<PathBuf>,
    include_hidden: bool,
    in_place: bool,
}

///Information required for a deryption command
#[derive(Debug)]
pub struct DecryptInfo {
    is_directory: bool,
    path: Vec<PathBuf>,
    in_place: bool,
}

///CLI arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineArgs {
    ///Enable debug mode
    #[arg(short, long)]
    pub debug: bool, //TODO: Implement debug

    ///TUI mode
    #[arg(short, long, default_value_t = false)]
    pub tui: bool,

    ///Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

///CLI commands
#[derive(Subcommand, Debug)]
enum Commands {
    ///Encrypt file or folder of files
    Encrypt {
        ///Path to File or Directory
        #[arg(required = true)]
        path: String,
        //Include hidden files
        #[arg(short = 'i', long, default_value_t = false)]
        include_hidden: bool,
        //Perform an in-place encryption
        #[arg(short = 'p', long, default_value_t = false)]
        in_place: bool,
    },
    ///Decrypt file or folder of files
    Decrypt {
        ///Path to File or Directory
        #[arg(required = true)]
        path: String,
        //Perform an in-place decryption
        #[arg(short = 'p', long, required = false)]
        in_place: bool,
    },
    ///Upload file or folder to cloud provider
    Upload {
        //TODO: Upload requirements and options
    },
    ///Change user config
    Config {
        //TODO: Configuration options
    },
}

///Runs the CLI and returns a directive to be processed
pub fn load_cli() -> anyhow::Result<()> {
    //Run the cli and get responses
    let cli = CommandLineArgs::parse();
    //If debug mode was passed
    if cli.debug {
        debug_mode()?;
    }

    // raise TUI if flag was passed
    if cli.tui {
        tui::load_tui()?;
    }

    match &cli.command {
        Some(Commands::Encrypt {
            path,
            include_hidden,
            in_place,
        }) => {
            match PathBuf::from(path).is_dir() {
                true => {
                    todo!();
                }
                // is a file
                false => {
                    // get filename, extension, and full path info
                    let fp = util::path::get_full_file_path(path).unwrap();
                    let contents: Vec<u8> = std::fs::read(&fp).unwrap();
                    let name = fp.file_name().unwrap();
                    let index = name.to_str().unwrap().find(".").unwrap();
                    let (filename, extension) = name.to_str().unwrap().split_at(index);

                    let mut fc =
                        encryption::FileCrypt::new(filename.to_string(), extension.to_string(), fp);

                    // generate key, nonce
                    fc.generate();

                    let mut encrypted_contents =
                        util::encryption::encryption(&mut fc, &contents).unwrap();

                    // prepend uuid to contents
                    encrypted_contents = parse::prepend_uuid(&fc.uuid, &mut encrypted_contents);

                    let mut crypt_file = format!("{}.crypt", fc.filename);

                    if *in_place {
                        crypt_file = format!("{}{}", fc.filename, fc.ext);
                    }
                    parse::write_contents_to_file(&crypt_file, encrypted_contents)
                        .expect("failed to write contents to file!");

                    //write fc to crypt_keeper
                    crypt_keeper::insert_crypt(&fc)
                        .expect("failed to insert FileCrypt data into database!");
                }
            };
            Ok(())
        }
        Some(Commands::Decrypt { path, in_place }) => {
            // let (is_directory, path) = process_path(&path)?;
            // Ok(Directive::Decrypt(DecryptInfo {
            //     is_directory,
            //     path,
            //     in_place: in_place.to_owned(),
            // }))
            dbg!(&path, &in_place);

            // get path to encrypted file
            let fp = util::path::get_full_file_path(path).unwrap();
            let contents: Vec<u8> = std::fs::read(&fp).unwrap();

            // rip out uuid from contents
            let (uuid, content) = contents.split_at(39);
            let uuid_str = String::from_utf8(uuid[0..36].to_vec()).unwrap();

            // query db with uuid
            let fc = crypt_keeper::query_crypt(uuid_str).unwrap();
            let file = format!("{}{}", fc.filename, fc.ext);

            // decrypt file
            let decrypted_content = encryption::decryption(fc, &content.to_vec()).expect("failed decryption");

            //write back to original file.ext
            //? if file exists, what do we do?
            if Path::new(&file).exists() {
                todo!();
            }
            write_contents_to_file(&file, decrypted_content).expect("failed writing content to file!");

            //? delete crypt file?
            std::fs::remove_file(path).expect("failed deleting .crypt file");

            Ok(())
        }
        Some(Commands::Upload {}) => {
            todo!();
        }
        Some(Commands::Config {}) => {
            todo!();
        }
        None => todo!(),
    }
}

fn debug_mode() -> anyhow::Result<()> {
    println!("Why would you do this ._.");

    return Ok(());
}
