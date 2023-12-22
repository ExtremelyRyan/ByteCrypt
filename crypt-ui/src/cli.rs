use clap::{Parser, Subcommand};
use crypt_core::{
    config::{self, ConfigTask, ItemsTask},
    token::{CloudService, CloudTask},
};

use crate::{
    directive::{self, Directive},
    tui::load_tui,
};

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
        ///Import CSV keeper file to database
        #[arg(short = 'i', long, required = false, default_value_t = false)]
        import: bool,

        ///Export Keeper to CSV file
        #[arg(short = 'e', long, required = false, default_value_t = false)]
        export: bool,

        //Path to CSV file for import
        #[arg(required = false, default_value_t = String::from(""))]
        csv_path: String,
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
            let directive = Directive::new(path.to_owned());
            directive.encrypt(in_place.to_owned(), output.to_owned());
        }

        // Decryption
        Some(Commands::Decrypt {
            path,
            in_place,
            output,
        }) => {
            let directive = Directive::new(path.to_owned());
            directive.decrypt(in_place.to_owned(), output.to_owned());
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
                let directive = Directive::new(path.to_owned());
                directive.cloud(CloudService::Google, task);
            }

            // Dropbox
            Some(CloudCommand::Dropbox { task }) => {
                let (task, path) = match task {
                    Some(DriveCommand::Upload { path }) => (CloudTask::Upload, path),
                    Some(DriveCommand::Download { path }) => (CloudTask::Download, path),
                    Some(DriveCommand::View { path }) => (CloudTask::View, path),
                    None => panic!("invalid input"),
                };
                let directive: Directive = Directive::new(path.to_owned());
                directive.cloud(CloudService::Dropbox, task);
            }

            None => {
                todo!();
            }
        },
        // Keeper
        Some(Commands::Keeper {
            import,
            export,
            csv_path,
        }) => {
            directive::Directive::keeper(import, export, csv_path);
        }

        // Config
        Some(Commands::Config { category }) => {
            match category {
                Some(ConfigCommand::DatabasePath { path }) => {
                    let directive = Directive::new(path.to_owned());
                    directive.config(ConfigTask::DatabasePath);
                }

                // IgnoreItems
                Some(ConfigCommand::IgnoreItems { add_remove, item }) => {
                    let add_remove = match add_remove.to_lowercase().as_str() {
                        "add" | "a" => ItemsTask::Add,
                        "remove" | "r" => ItemsTask::Remove,
                        _ => panic!("invalid input"),
                    };

                    let directive = Directive::default();
                    directive.config(ConfigTask::IgnoreItems(add_remove, item.to_owned()));
                }

                // Retain
                Some(ConfigCommand::Retain { choice }) => {
                    let directive = Directive::default();
                    let choice = match choice.to_lowercase().as_str() {
                        "true" | "t" => true,
                        "false" | "f" => false,
                        _ => panic!("Unable to parse passed value"),
                    };
                    directive.config(ConfigTask::Retain(choice));
                }

                // Backup
                Some(ConfigCommand::Backup { choice }) => {
                    let directive = Directive::default();
                    let choice = match choice.to_lowercase().as_str() {
                        "true" | "t" => true,
                        "false" | "f" => false,
                        _ => panic!("Unable to parse passed value"),
                    };
                    directive.config(ConfigTask::Backup(choice));
                }

                // ZstdLevel
                Some(ConfigCommand::ZstdLevel { level }) => {
                    let directive = Directive::default();
                    let level: i32 = level.parse().expect("Could not interpret passed value");
                    directive.config(ConfigTask::ZstdLevel(level));
                }

                // LoadDefault
                Some(ConfigCommand::LoadDefault) => {
                    let directive = Directive::default();
                    directive.config(ConfigTask::LoadDefault);
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
