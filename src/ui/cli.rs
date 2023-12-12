use std::path::PathBuf;

use super::tui;
use crate::util::{config::Config, directive::{Directive, self}, directive::*};
use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};

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
pub fn load_cli(config: Config) -> anyhow::Result<()> {
    //Run the cli and get responses
    let cli = CommandLineArgs::parse();

    //If debug mode was passed
    if cli.debug {
        debug_mode();
    }

    //Call TUI if flag was passed
    if cli.tui {
        tui::load_tui()?;
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
            Ok(())
        }
        //Decryption
        Some(Commands::Decrypt { path, output }) => {
            Directive::process_directive(Directive::Decrypt(DecryptInfo {
                path: path.to_owned(),
                output: output.to_owned(),
                config,
            }));
            Ok(())
        }
        //Upload
        Some(Commands::Upload {}) => {
            todo!();
        }
        //Config
        Some(Commands::Config {
            update,
            value,
            value2,
        }) => {
            //Show the config
            //? not sure how i feel about this, atm I want these to keep seperate.
            println!("{:#?}", config);

            //Check for if update passed
            let fields = Config::get_fields();
            match fields.contains(&update.as_str()) {
                true => Directive::process_directive(Directive::Config(ConfigInfo {
                    update: update.to_owned(),
                    value: value.to_owned(),
                    value2: value2.to_owned(),
                    config,
                })),
                false => (),
            };
            Ok(())
        }
        //Nothing passed (Help screen printed)
        None => Ok(()),
    }
}

fn debug_mode() {
    println!("Why would you do this ._.");
}
