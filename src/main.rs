mod cloud_storage;
mod ui;
mod util;
mod database;
use anyhow::{self, Ok, Result};
use util::*;
use ui::*;
use database::*;


fn main() -> Result<()> {
    //Load config file
    config::load_config();

    //Load the UI - CLI only currently
    //let _ = ui::cli::load_cli();
    let _ = tui::load_tui();  //Uncomment for TUI

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
