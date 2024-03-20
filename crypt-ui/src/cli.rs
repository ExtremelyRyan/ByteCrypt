use std::path::PathBuf;

use clap::{Parser, Subcommand};
use crypt_cloud::crypt_core::{
    common::{get_machine_name, send_information},
    config::{self, ConfigTask, ItemsTask},
    db::import_keeper,
};

use crate::directive::{
    self, dropbox_download, dropbox_upload, dropbox_view, google_download, google_view,
};
// use crate::tui::load_tui;

///CLI arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct CommandLineArgs {
    ///Enable debug mode
    #[arg(short, long)]
    pub debug: bool, //TODO: Implement debug needed?

    /// generate markdown document for commands
    #[arg(long, hide = true)]
    md: bool,

    ///TUI mode
    // #[arg(short, long, default_value_t = false)]
    // pub tui: bool,

    #[arg(short, default_value_t = false)]
    pub test: bool,

    ///Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

///CLI commands
#[derive(Subcommand, Debug)]
enum Commands {
    ///Upload, download, or view file or folder to cloud provider
    Cloud {
        ///Categories
        #[command(subcommand)]
        category: Option<CloudCommand>,
    },

    /// View or change configuration
    Config {
        /// Categories
        #[command(subcommand)]
        category: Option<ConfigCommand>,
    },

    ///Encrypt file or folder of files
    Encrypt {
        ///Path to File or Directory
        #[arg(required = true)]
        path: String,

        ///Change the output path
        #[arg(short = 'o', long, required = false)]
        output: Option<String>,
    },

    ///Decrypt file or folder of files
    Decrypt {
        ///Path to File or Directory
        #[arg(required = false, default_value_t = String::from(""))]
        path: String,

        ///Change the output path
        #[arg(short = 'o', long, required = false)]
        output: Option<String>,
    },

    ///Import | Export | Purge database
    Keeper {
        /// Categories
        #[command(subcommand)]
        category: Option<KeeperCommand>,
    },

    /// show local / cloud crypt folder
    Ls {
        ///Show all files contained in the local crypt folder
        #[arg(short = 'l', long, default_value_t = false)]
        local: bool,

        ///Show all files contained in the cloud folder
        #[arg(short = 'c', long, default_value_t = false)]
        cloud: bool,
    },
}

///Subcommands for Upload
#[derive(Subcommand, Debug)]
pub enum CloudCommand {
    /// View, upload, or download actions for Google Drive
    #[command(short_flag = 'g')]
    Google {
        #[command(subcommand)]
        task: Option<DriveCommand>,
    },

    /// View, upload, or download actions for DropBox
    #[command(short_flag = 'd')]
    Dropbox {
        #[command(subcommand)]
        task: Option<DriveCommand>,
    },
}

///
#[derive(Subcommand, Debug, Clone)]
pub enum DriveCommand {
    /// Upload a file or folder
    #[command(short_flag = 'u')]
    Upload {
        // /// Path to the file to be encrypted and uploaded to the cloud
        // #[arg(required = false, default_value_t = String::from(""))]
        // path: String,
        // /// if flag is passed, do not encrypt.
        // #[arg(long, short)]
        // no_encrypt: bool,
    },

    /// Download a file or folder
    #[command(short_flag = 'd')]
    Download {
        /// name of the file you want to get from the cloud
        #[arg(required = false, default_value_t = String::from(""))]
        path: String,
    },

    /// View a file or folder
    #[command(short_flag = 'v')]
    #[clap(alias = "list")]
    View {
        // Default to Crypt folder if nothing passed in.
        #[arg(required = false, default_value_t = String::from("Crypt"))]
        path: String,
    },
}

/// Subcommands for Config
#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// View or update the database path
    #[command(short_flag = 'd')]
    DatabasePath {
        /// Database path; if empty, prints current path
        #[arg(required = false, default_value_t = String::from(""))]
        path: String,
    },

    /// View or update the crypt folder path
    #[command(short_flag = 'c')]
    CryptPath {
        /// Database path; if empty, prints current path
        #[arg(required = false, default_value_t = String::from(""))]
        path: String,
    },

    /// View or change which directories and/or filetypes are to be ignored
    #[command(short_flag = 'i')]
    IgnoreItems {
        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        add_remove: String,

        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        item: String,
    },

    /// View or change current pc name associated with the cloud.
    #[command()]
    Hwid {},

    /// View or change the compression level (-7 to 22) -- higher is more compression
    #[command(short_flag = 'z')]
    ZstdLevel {
        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        level: String,
    },

    /// Revert config back to default
    #[command(short_flag = 'l')]
    LoadDefault,
}

/// Subcommands for Keeper
#[derive(Subcommand, Debug)]
pub enum KeeperCommand {
    /// View or update the database path
    #[command(short_flag = 'i')]
    Import {
        #[arg(required = true, default_value_t = String::from(""))]
        path: String,
    },

    /// View or change which directories and/or filetypes are to be ignored
    #[command(short_flag = 'e')]
    Export {
        /// value to update config
        #[arg(required = false, default_value_t = String::from(""))]
        alt_path: String,
    },

    /// PURGES DATABASE FROM SYSTEM
    #[command(short_flag = 'p')]
    Purge {
        /// Categories
        #[command(subcommand)]
        category: Option<KeeperPurgeSubCommand>,
    },
    // TODO: maybe get rid of this in the future. for now, handy debugging tool for small db.
    /// List each file in the database
    #[command(short_flag = 'l')]
    List {},
}

/// Subcommands for Keeper
#[derive(Subcommand, Debug)]
pub enum KeeperPurgeSubCommand {
    /// Purges google and Dropbox tokens
    #[command(short_flag = 't', alias = "tokens")]
    Token {},

    /// Purges database file and IS UNREVERSABLE!
    #[command(short_flag = 'd', alias = "db")]
    Database {},
}

impl KeeperCommand {
    pub fn import(path: &String) {
        if path.is_empty() {
            send_information(vec![format!("please add a path to the csv")]);
            return;
        }
        match import_keeper(path) {
            Ok(_) => (),
            Err(e) => panic!("problem importing keeper to database! {}", e),
        }
    }
}

/// Runs the CLI and returns a directive to be processed
pub fn load_cli() {
    config::init(config::Interface::CLI);

    // Run the cli and get responses
    let cli = CommandLineArgs::parse();

    // Invoked as: `crypt --md > commands.md`
    if cli.md {
        clap_markdown::print_help_markdown::<CommandLineArgs>();
    }

    // If debug mode was passed
    if cli.debug {
        debug_mode();
    }

    // Call TUI if flag was passed
    // if cli.tui {
    //     // load_tui().expect("failed to load TUI");
    // }

    if cli.test {
        directive::test();
    }

    // Process the command passed by the user
    match &cli.command {
        // Nothing passed (Help screen printed)
        None => (),

        // ls
        Some(Commands::Ls { local, cloud }) => {
            directive::ls(local, cloud);
        }

        // Encryption
        Some(Commands::Encrypt { path, output }) => {
            let res = directive::encrypt(path, output.to_owned());
            println!("encrypt result: {:?}", res);
        }

        // Decryption
        Some(Commands::Decrypt { path, output }) => {
            directive::decrypt(path, output.to_owned());
        }

        // Cloud commands - upload | download | view for Google Drive and TODO: Dropbox
        Some(Commands::Cloud { category }) => match category {
            // Google
            Some(CloudCommand::Google { task }) => {
                match task {
                    Some(DriveCommand::Upload {}) => {
                        let response = directive::google_upload();
                        if let Err(e) = response {
                            println!("error: {}", e);
                        }
                    }
                    Some(DriveCommand::Download { path }) => {
                        let response = google_download(path);
                        if let Err(e) = response {
                            println!("error: {}", e);
                        }
                    }
                    Some(DriveCommand::View { path }) => _ = google_view(path),
                    None => panic!("invalid input"),
                };
            }

            // Dropbox
            // TODO:
            Some(CloudCommand::Dropbox { task }) => {
                match task {
                    Some(DriveCommand::Upload {}) => dropbox_upload(""),
                    Some(DriveCommand::Download { path }) => dropbox_download(path),
                    Some(DriveCommand::View { path }) => dropbox_view(path),
                    None => panic!("invalid input"),
                };
            }

            None => {}
        },
        // Keeper
        Some(Commands::Keeper { category }) => {
            let kc = category.as_ref().unwrap();
            directive::keeper(kc);
        }

        // Config
        Some(Commands::Config { category }) => {
            match category {
                Some(ConfigCommand::DatabasePath { path }) => {
                    directive::config(path, ConfigTask::DatabasePath);
                }

                Some(ConfigCommand::CryptPath { path }) => {
                    directive::config(path, ConfigTask::CryptPath);
                }

                // IgnoreItems
                Some(ConfigCommand::IgnoreItems { add_remove, item }) => {
                    let add_remove = match add_remove.to_lowercase().as_str() {
                        "add" | "a" => ItemsTask::Add,
                        "remove" | "r" => ItemsTask::Remove,
                        _ => panic!("invalid input"),
                    };

                    directive::config("", ConfigTask::IgnoreItems(add_remove, item.to_owned()));
                }

                // ZstdLevel
                Some(ConfigCommand::ZstdLevel { level }) => {
                    let level: i32 = level.parse().expect("Could not interpret passed value");
                    directive::config("", ConfigTask::ZstdLevel(level));
                }

                //Hwid
                Some(ConfigCommand::Hwid {}) => {
                    send_information(vec![format!("machine name: {}", get_machine_name())]);
                }

                // LoadDefault
                Some(ConfigCommand::LoadDefault) => {
                    directive::config("", ConfigTask::LoadDefault);
                }

                None => (),
            }
            // let config = config::get_config();
            // println!("{}", config);
        }
    }
}

fn debug_mode() {
    println!("Why would you do this ._.");
}

pub fn test() {
    // let crypts = crypt_cloud::crypt_core::db::query_keeper_for_files_with_drive_id().unwrap();

    // for crypt in crypts {
    //     println!("file: {}{}", crypt.filename, crypt.ext);
    //     println!("full path: {}", crypt.full_path.display());
    //     println!("drive ID: {}\n", crypt.drive_id);
    // }

    // Get the current working directory
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    // Specify the file or directory for which you want to find the relative path
    let target_path = "test_folder\\folder2\\file3.txt";

    // Create a PathBuf for the target path
    let target_path_buf = PathBuf::from(target_path);

    // Resolve the full path of the target path
    let full_path = current_dir.join(target_path_buf);

    // Get the relative path from the current directory to the target path
    let relative_path = full_path
        .strip_prefix(&current_dir)
        .expect("Failed to calculate relative path");

    println!("Current Directory: {:?}", current_dir);
    println!("Full Path: {:?}", full_path);
    println!("Relative Path: {:?}", relative_path);
}
