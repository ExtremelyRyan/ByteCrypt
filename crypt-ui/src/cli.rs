use clap::{Parser, Subcommand};
use crypt_cloud::crypt_core::{
    common::send_information,
    config::{self, ConfigTask, ItemsTask},
    db::import_keeper,
    token::{CloudService, CloudTask},
};

use crate::directive;
use crate::tui::load_tui;

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
    #[arg(short, long, default_value_t = false)]
    pub tui: bool,

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

        ///Perform an in-place encryption
        #[arg(short = 'p', long, default_value_t = false)]
        in_place: bool,

        ///Change the output path
        #[arg(short = 'o', long, required = false)]
        output: Option<String>,
    },

    ///Decrypt file or folder of files
    Decrypt {
        ///Path to File or Directory
        #[arg(required = true)]
        path: String,

        ///Perform an in-place decryption
        #[arg(short = 'p', long, default_value_t = false)]
        in_place: bool,

        ///Change the output path
        #[arg(short = 'o', long, required = false)]
        output: Option<String>,
    },

    ///Import | Export database
    Keeper {
        /// Categories
        #[command(subcommand)]
        category: Option<KeeperCommand>,
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
        #[arg(required = false, default_value_t = String::from(""))]
        path: String,
    },

    /// Download a file or folder
    #[command(short_flag = 'd')]
    Download {
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

    /// Update whether to retain original files after encryption or decryption
    #[command(short_flag = 'r')]
    Retain {
        /// Configure retaining original file: kept if true
        #[arg(required = false, default_value_t = String::from(""))]
        choice: String,
    },

    /// Update whether to retain original files after encryption or decryption
    #[command(short_flag = 'b')]
    Backup {
        /// Configure retaining original file: kept if true
        #[arg(required = false, default_value_t = String::from(""))]
        choice: String,
    },

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
        /// tokens | db
        #[arg(required = true, default_value_t = String::from(""))]
        item: String,
    },
    /// TODO: maybe get rid of this in the future. for now, handy debugging tool for small db.
    /// List each file in the database
    #[command(short_flag = 'l')]
    List {},
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
    if cli.tui {
        load_tui().expect("failed to load TUI");
    }

    // Process the command passed by the user
    match &cli.command {
        // Nothing passed (Help screen printed)
        None => (),

        // Encryption
        Some(Commands::Encrypt {
            path,
            in_place,
            output,
        }) => {
            directive::encrypt(path, in_place.to_owned(), output.to_owned());
        }

        // Decryption
        Some(Commands::Decrypt {
            path,
            in_place,
            output,
        }) => {
            directive::decrypt(path, in_place.to_owned(), output.to_owned());
        }

        // Cloud
        Some(Commands::Cloud { category }) => match category {
            Some(CloudCommand::Google { task }) => {
                let (task, path) = match task {
                    Some(DriveCommand::Upload { path }) => (CloudTask::Upload, path),
                    Some(DriveCommand::Download { path }) => (CloudTask::Download, path),
                    Some(DriveCommand::View { path }) => (CloudTask::View, path),
                    None => panic!("invalid input"),
                };
                directive::cloud(path, CloudService::Google, task);
            }

            // Dropbox
            Some(CloudCommand::Dropbox { task }) => {
                let (task, path) = match task {
                    Some(DriveCommand::Upload { path }) => (CloudTask::Upload, path),
                    Some(DriveCommand::Download { path }) => (CloudTask::Download, path),
                    Some(DriveCommand::View { path }) => (CloudTask::View, path),
                    None => panic!("invalid input"),
                };
                directive::cloud(path, CloudService::Dropbox, task);
            }

            None => {
                todo!();
            }
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

                // IgnoreItems
                Some(ConfigCommand::IgnoreItems { add_remove, item }) => {
                    let add_remove = match add_remove.to_lowercase().as_str() {
                        "add" | "a" => ItemsTask::Add,
                        "remove" | "r" => ItemsTask::Remove,
                        _ => panic!("invalid input"),
                    };

                    directive::config("", ConfigTask::IgnoreItems(add_remove, item.to_owned()));
                }

                // Retain
                Some(ConfigCommand::Retain { choice }) => {
                    let choice = match choice.to_lowercase().as_str() {
                        "true" | "t" => true,
                        "false" | "f" => false,
                        _ => panic!("Unable to parse passed value"),
                    };
                    directive::config("", ConfigTask::Retain(choice));
                }

                // Backup
                Some(ConfigCommand::Backup { choice }) => {
                    let choice = match choice.to_lowercase().as_str() {
                        "true" | "t" => true,
                        "false" | "f" => false,
                        _ => panic!("Unable to parse passed value"),
                    };
                    directive::config("", ConfigTask::Backup(choice));
                }

                // ZstdLevel
                Some(ConfigCommand::ZstdLevel { level }) => {
                    let level: i32 = level.parse().expect("Could not interpret passed value");
                    directive::config("", ConfigTask::ZstdLevel(level));
                }

                // LoadDefault
                Some(ConfigCommand::LoadDefault) => {
                    directive::config("", ConfigTask::LoadDefault);
                }

                None => (),
            }
            let config = config::get_config();
            println!("{}", config);
        }
    }
}

fn debug_mode() {
    println!("Why would you do this ._.");
}
