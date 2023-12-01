use super::tui;
use crate::{
    database::crypt_keeper,
    util::{
        self,
        config::Config,
        encryption::{decrypt_file, encrypt_file},
        parse::write_contents_to_file,
        *,
    },
};
use anyhow::{Ok, Result};
use blake2::{Blake2s256, Digest};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

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
    ///Change user config
    Config {
        //TODO: Configuration options
    },
}

///Runs the CLI and returns a directive to be processed
pub fn load_cli(conf: Config) -> anyhow::Result<()> {
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
                    todo!();
                }
                // is a file
                false => {
                    encrypt_file(conf, path, *in_place);
                }
            };
            Ok(())
        }
        Some(Commands::Decrypt { path, output }) => {
            let _res = decrypt_file(conf, path, output.to_owned());
            Ok(())
        }

        Some(Commands::Upload {}) => {
            todo!();
        }

        Some(Commands::Config {}) => {
            todo!();
        }
        None => Ok(()),
    }
}

fn debug_mode() {
    println!("Why would you do this ._.");
}
