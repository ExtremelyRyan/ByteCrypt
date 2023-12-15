use super::tui;
use crate::util::{config::Config, directive::*};
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

    ///Upload, download, or view file or folder to cloud provider
    Cloud {
        ///Categories
        #[command(subcommand)]
        category: Option<CloudCommand>,
    },

    ///View or change configuration
    Config {
        ///Categories
        #[command(subcommand)]
        category: Option<ConfigCommand>,
    },
}

///Subcommands for Upload
#[derive(Subcommand, Debug)]
pub enum CloudCommand {
    ///View, upload, or download actions for Google Drive
    #[command(short_flag = 'g')]
    Google {
        #[command(subcommand)]
        task: Option<DriveCommand>,
    },

    ///View, upload, or download actions for DropBox
    #[command(short_flag = 'd')]
    Dropbox {
        #[command(subcommand)]
        task: Option<DriveCommand>,
    },
}

///
#[derive(Subcommand, Debug, Clone)]
pub enum DriveCommand {
    ///Upload a file or folder
    #[command(short_flag = 'u')]
    Upload {
        #[arg(required = false, default_value_t = String::from(""))]
        path: String,
    },

    ///Download a file or folder
    #[command(short_flag = 'd')]
    Download {
        #[arg(required = false, default_value_t = String::from(""))]
        path: String,
    },

    ///View a file or folder
    #[command(short_flag = 'v')]
    View {
        #[arg(required = false, default_value_t = String::from(""))]
        path: String,
    },
}

///Subcommands for Config
#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    ///View or update the database path
    #[command(short_flag = 'd')]
    DatabasePath {
        ///Database path; if empty, prints current path
        #[arg(required = false, default_value_t = String::from(""))]
        path: String,
    },

    ///Update whether to retain original files after encryption or decryption
    #[command(short_flag = 'r')]
    Retain {
        ///Configure retaining original file: kept if true
        #[arg(required = false, default_value_t = String::from(""))]
        value: String,
    },

    ///View or change which directories and/or filetypes are to be ignored
    #[command(short_flag = 'i')]
    IgnoreDirectories {
        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        value: String,

        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        value2: String,
    },

    ///View or change the compression level (-7 to 22) -- higher is more compression
    #[command(short_flag = 'z')]
    ZstdLevel {
        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        value: String,
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
        //Cloud
        Some(Commands::Cloud { category }) => match category {
            Some(CloudCommand::Google { task }) => {
                let (tsk, pth) = match task {
                    Some(DriveCommand::Upload { path }) => (CloudTask::Upload, path.to_owned()),
                    Some(DriveCommand::Download { path }) => (CloudTask::Download, path.to_owned()),
                    Some(DriveCommand::View { path }) => (CloudTask::View, path.to_owned()),
                    None => (CloudTask::View, "".to_owned()),
                };
                Directive::process_directive(Directive::Cloud(CloudInfo {
                    platform: CloudPlatform::Google,
                    task: tsk,
                    path: pth,
                    config,
                }));
                Ok(())
            }
            Some(CloudCommand::Dropbox { task }) => {
                let (tsk, pth) = match task {
                    Some(DriveCommand::Upload { path }) => (CloudTask::Upload, path.to_owned()),
                    Some(DriveCommand::Download { path }) => (CloudTask::Download, path.to_owned()),
                    Some(DriveCommand::View { path }) => (CloudTask::View, path.to_owned()),
                    None => (CloudTask::View, "".to_owned()),
                };
                Directive::process_directive(Directive::Cloud(CloudInfo {
                    platform: CloudPlatform::DropBox,
                    task: tsk,
                    path: pth,
                    config,
                }));
                Ok(())
            }
            None => {
                //TODO: print out default info?
                todo!();
            }
        },
        //Config
        Some(Commands::Config { category }) => match category {
            Some(ConfigCommand::DatabasePath { path: value }) => {
                Directive::process_directive(Directive::Config(ConfigInfo {
                    category: String::from("database_path"),
                    value: value.to_owned(),
                    value2: String::from(""),
                    config,
                }));
                Ok(())
            }
            Some(ConfigCommand::Retain { value }) => {
                Directive::process_directive(Directive::Config(ConfigInfo {
                    category: String::from("retain"),
                    value: value.to_owned(),
                    value2: String::from(""),
                    config,
                }));
                Ok(())
            }
            Some(ConfigCommand::IgnoreDirectories { value, value2 }) => {
                Directive::process_directive(Directive::Config(ConfigInfo {
                    category: String::from("ignore_directories"),
                    value: value.to_owned(),
                    value2: value2.to_owned(),
                    config,
                }));
                Ok(())
            }
            Some(ConfigCommand::ZstdLevel { value }) => {
                Directive::process_directive(Directive::Config(ConfigInfo {
                    category: String::from("zstd_level"),
                    value: value.to_owned(),
                    value2: String::from(""),
                    config,
                }));
                Ok(())
            }
            None => {
                println!("Current config: \n{}", config);
                Ok(())
            }
        },
        //Nothing passed (Help screen printed)
        None => Ok(()),
    }
}

fn debug_mode() {
    println!("Why would you do this ._.");
}
