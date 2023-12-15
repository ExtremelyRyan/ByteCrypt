use super::tui;
use crate::util::{
    config::Config,
    directive::*,
    directive::{self, Directive},
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

        ///Perform an in-place decryption
        #[arg(short = 'o', long, required = false)]
        output: Option<String>,
    },

    ///Upload file or folder to cloud provider
    Upload {
        //TODO: Upload requirements and options
    },

    ///View or change configuration
    Config {
        ///Categories
        #[command(subcommand)]
        category: Option<ConfigCommand>,
    },
}

///Subcommands for Config
#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    ///View or update the database path
    DatabasePath {
        ///Database path; if empty, prints current path
        #[arg(required = false, default_value_t = String::from(""))]
        path: String,
    },

    ///Update whether to retain original files after encryption or decryption
    Retain {
        ///Configure retaining original file: kept if true
        #[arg(required = false, default_value_t = String::from(""))]
        value: String,
    },

    ///View or change which directories and/or filetypes are to be ignored
    IgnoreDirectories {
        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        value: String,

        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        value2: String,
    },

    ///View or change the compression level (-7 to 22) -- higher is more compression
    ZstdLevel {
        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        value: String,
    },
}

///Runs the CLI and returns a directive to be processed
pub fn load_cli(config: Config) {
    //Run the cli and get responses
    let cli = CommandLineArgs::parse();

    //If debug mode was passed
    if cli.debug {
        debug_mode();
    }

    //Call TUI if flag was passed
    if cli.tui {
        tui::load_tui().expect("failed to load TUI");
    }

    //Process the command passed by the user
    match &cli.command {
        //Encryption
        Some(Commands::Encrypt { path, in_place }) => {
            Directive::process_directive(Directive::Encrypt(EncryptInfo {
                path: path.to_owned(),
                in_place: in_place.to_owned(),
                config,
            }));
        }
        //Decryption
        Some(Commands::Decrypt { path, output }) => {
            Directive::process_directive(Directive::Decrypt(DecryptInfo {
                path: path.to_owned(),
                output: output.to_owned(),
                config,
            }));
        }
        //Upload
        Some(Commands::Upload {}) => {
            todo!();
        }
        //Config
        Some(Commands::Config { category }) => match category {
            Some(ConfigCommand::DatabasePath { path: value }) => {
                Directive::process_directive(Directive::Config(ConfigInfo {
                    category: String::from("database_path"),
                    value: value.to_owned(),
                    value2: String::from(""),
                    config,
                }));
            }
            Some(ConfigCommand::Retain { value }) => {
                Directive::process_directive(Directive::Config(ConfigInfo {
                    category: String::from("retain"),
                    value: value.to_owned(),
                    value2: String::from(""),
                    config,
                }));
            }
            Some(ConfigCommand::IgnoreDirectories { value, value2 }) => {
                Directive::process_directive(Directive::Config(ConfigInfo {
                    category: String::from("ignore_directories"),
                    value: value.to_owned(),
                    value2: value2.to_owned(),
                    config,
                }));
            }
            Some(ConfigCommand::ZstdLevel { value }) => {
                Directive::process_directive(Directive::Config(ConfigInfo {
                    category: String::from("zstd_level"),
                    value: value.to_owned(),
                    value2: String::from(""),
                    config,
                }));
            }
            None => {
                println!("{}", config);
            }
        },
        //Nothing passed (Help screen printed)
        None =>(),
    }
}

fn debug_mode() {
    println!("Why would you do this ._.");
}
