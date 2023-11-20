mod cloud_storage;
mod ui;
mod util;
use anyhow::{self, Ok, Result};
use ui::ui::load_ui;
use util::*;

fn main() -> Result<()> {
    config::load_config();

    //Load the UI - CLI only currently
    load_ui();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_path() {
        let dir = "../test_folder";
        for p in path::walk_directory(dir).unwrap() {
            let s = util::common::read_to_vec_string(p.as_str());
            println!("{:?} from file: {}", s, p);
        }
    }
}

fn _test_write_db() -> Result<()> {
    let t = util::parse::toml_example()?;
    println!("{:?}", t);
    parse::prepend_file(t, "db")
}
