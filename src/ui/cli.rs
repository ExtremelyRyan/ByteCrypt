use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CommandLineInterface {
    //Debug
    #[arg(short, long)]
    pub debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    ///Encrypt file or directory
    Encrypt {
        ///File or Directory
        #[arg(required = true)]
        structure: String,
        ///Path to the given File or Directory
        #[arg(required = true)]
        path: String,
        ///Name of the File or Directory
        #[arg(required = true)]
        name: String,
        #[arg(short, long, required = true)]
        copy: bool,
    },
    ///Decrypt file or folder
    Decrypt {
        ///File or Directory
        #[arg(required = true)]
        structure: String,
        ///Path to File or Directory
        #[arg(required = true)]
        path: String,
        ///Name of File or Directory
        #[arg(required = true)]
        name: String,
    },
    ///Upload file or folder to cloud provider
    Upload {
        //TODO: Upload requirements and options
    },
    ///Change user configuration
    ///Default used if not specified or changed
    Preferences {
        //TODO: Configuration options
    },
}

pub fn load_cli() -> anyhow::Result<()> {
    let command_line = CommandLineInterface::parse();
    //Choose copy or in place
    //File or directory
    //Directory or file path
    //Zip

    match &command_line.command {
        Some(Commands::Encrypt {
            structure: _,
            path: _,
            name: _,
            copy: _,
        }) => {
            todo!();
        }
        Some(Commands::Decrypt {
            structure: _,
            path: _,
            name: _,
        }) => {
            todo!();
        }
        Some(Commands::Upload {}) => {
            todo!();
        }
        Some(Commands::Preferences {}) => {
            todo!();
        }
        None => todo!(),
    }
}
