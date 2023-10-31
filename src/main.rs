use anyhow::{self, Result};
mod util;

fn main() -> Result<()> {
    // let dir = "../test_folder";

    // let paths = walk_directory(dir).unwrap();

    // for p in paths {
    //     let s = util::common::read_to_vec_string(p.as_str());
    //     println!("{:?} from file: {}", s, p);
    // }

    // let f = json_example().unwrap();

    let t = util::parse::toml_example().unwrap();

    // println!("{:?}", f);
    println!("{:?}", t);
    util::parse::prepend_file(t, "db.txt");

    Ok(())
}
