use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use anyhow::Ok;
use crate::util::*;



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
pub fn load_cli() -> anyhow::Result<Directive> {
    //Run the cli and get responses
    let cli = CommandLineArgs::parse();
    //If debug mode was passed
    if cli.debug { debug_mode()?; }
    
    match &cli.command {
        Some(Commands::Encrypt {
            path,
            include_hidden,
            in_place,
        }) => {
            let (is_directory, path) = process_path(&path)?;
            Ok(Directive::Encrypt(EncryptInfo {
                is_directory,
                path,
                include_hidden: include_hidden.to_owned(),
                in_place: in_place.to_owned(),
            }))
        }
        Some(Commands::Decrypt {
            path,
            in_place,
        }) => {
            let (is_directory, path) = process_path(&path)?;
            Ok(Directive::Decrypt(DecryptInfo {
                is_directory,
                path,
                in_place: in_place.to_owned(),
            }))
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

///Determines if valid path, returns if is_dir boolean and full filepath
fn process_path(path_in: &str) -> anyhow::Result<(bool, Vec<PathBuf>)> {
    //Determine the path
    let is_directory = Path::new(path_in).is_dir();
    let path = path::walk_directory(path_in);

    return Ok((is_directory, path.unwrap()));
}

fn debug_mode() -> anyhow::Result<()> {
    println!("Why would you do this ._.");

    return Ok(());
}
