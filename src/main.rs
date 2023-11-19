
mod util;
use anyhow::{self, Result, Ok};
use util::*; 


fn main() -> Result<()> {

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reading_path(){
        let dir = "../test_folder";

        let paths = path::walk_directory(dir).unwrap();
    
        for p in paths {
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