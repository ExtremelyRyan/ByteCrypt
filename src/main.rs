mod cloud_storage;
mod ui;
mod util;

use anyhow::{self, Ok, Result};
use util::*;

fn main() -> Result<()> {
    //Load config file
    config::load_config();

    //Load the UI - CLI only currently
    let _ = ui::cli::load_cli();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_path() {
        let dir = "./test_folder_backup";
        for p in path::walk_directory(dir).unwrap() {
            let s = util::common::read_to_vec_string(p.as_str());
            println!("{:?} from file: {}", s, p);
        }
    }
}
