use super::tui;
use crate::{
    database::crypt_keeper,
    util::{
        self,
        config::Config,
        encryption::{decrypt_file, encrypt_file},
        parse::write_contents_to_file,
        path::{get_full_file_path, walk_directory},
        *,
    },
};
use anyhow::{Ok, Result};
use blake2::{Blake2s256, Digest};
use clap::{builder::OsStr, Parser, Subcommand};
use std::path::{Path, PathBuf};

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

#[derive(Debug)]
pub struct UploadInfo {
    placeholder: bool,
}

#[derive(Debug)]
pub struct ConfigInfo {
    placeholder: bool,
}

///CLI arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct CommandLineArgs {
    ///Enable debug mode
    #[arg(short, long)]
    pub debug: bool, //TODO: Implement debug needed?

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
        #[arg(short = 'o', long, required = false)]
        output: Option<String>,
    },
    ///Upload file or folder to cloud provider
    Upload {
        //TODO: Upload requirements and options
    },
    ///View or change configuration
    Config {
        /// show saved configuration options
        #[arg(short = 's', long, required = false)]
        show: bool,

        /// select config parameter to update
        #[arg(short = 'u', long, required = false, default_value_t = String::from(""))]
        update: String,

        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        value: String,
        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        value2: String,
    },
}

///Runs the CLI and returns a directive to be processed
pub fn load_cli(mut conf: Config) -> anyhow::Result<()> {
    //Run the cli and get responses
    let cli = CommandLineArgs::parse();

    //If debug mode was passed
    if cli.debug {
        debug_mode();
    }

    // raise TUI if flag was passed
    if cli.tui {
        tui::load_tui()?;
    }

    match &cli.command {
        Some(Commands::Encrypt { path, in_place }) => {
            match PathBuf::from(path).is_dir() {
                true => {
                    // get vec of dir
                    let dir = walk_directory(path, &conf).expect("could not find directory!");
                    // dbg!(&dir);
                    for path in dir {
                        println!("Encrypting file: {}", path.display());
                        encrypt_file(
                            &conf,
                            path.display().to_string().as_str(),
                            in_place.to_owned(),
                        )
                    }
                }
                // is a file
                false => {
                    encrypt_file(&conf, path, *in_place);
                }
            };
            Ok(())
        }
        Some(Commands::Decrypt { path, output }) => {
            match PathBuf::from(path).is_dir() {
                true => {
                    // get vec of dir
                    let dir = walk_directory(path, &conf).expect("could not find directory!");
                    // dbg!(&dir);
                    for path in dir {
                        if path.extension() == Some(&OsStr::from("crypt")) {
                            println!("Decrypting file: {}", path.display());
                            let _ = decrypt_file(
                                &conf,
                                path.display().to_string().as_str(),
                                output.to_owned(),
                            );
                        }
                    }
                }
                // is a file
                false => {
                    let _ = decrypt_file(&conf, path, output.to_owned());
                }
            };

            Ok(())
        }

        Some(Commands::Upload {}) => {
            todo!();
        }

        Some(Commands::Config {
            show,
            update,
            value,
            value2,
        }) => {
            if *show {
                println!("{}", conf);
                //? not sure how i feel about this, atm I want these to keep seperate.
                return Ok(());
            };
            let fields = Config::get_fields();

            if fields.contains(&update.as_str()) {
                if value.is_empty() {
                    println!("cannot update {}, missing update value", update);
                    return Ok(()); // TODO: fix this later
                }
                match update.as_str() {
                    // TODO set path
                    "database_path" => match value.to_lowercase().as_str() {
                        "get" | "g" => println!(
                            "database_path: {}",
                            get_full_file_path(conf.get_database_path())?.display()
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
                                if PathBuf::from(value2).exists() {
                                    conf.set_database_path(value2);
                                } else {
                                    // create path
                                }
                                conf.set_database_path(value2);
                            }
                        }
                        _ => println!("not valid"),
                    },
                    // TODO: add / remove items in list
                    // "cloud_services" => todo!(),
                    "retain" => match conf.set_retain(value.to_owned()) {
                        false => eprintln!("Error occured, please verify parameters."),
                        true => println!("{} value changed to: {}", update, value),
                    },
                    "hidden_directories" => match value.to_lowercase().as_str() {
                        "add" | "a" => conf.append_ignore_directories(value2),
                        "remove" | "r" => conf.remove_item_from_ignore_directories(value2),
                        _ => println!("invalid input"),
                    },
                    _ => eprintln!(
                        "invalid selection!\n use -s | --show to see available config options."
                    ),
                }
            }

            Ok(())
        }
        // todo: Find some way to print the help screen if nothing is passed.
        None => Ok(()),
    }
}

fn debug_mode() {
    println!("Why would you do this ._.");
}
