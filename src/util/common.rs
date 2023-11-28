use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::util;

/// read file, and return values within a Vector of Strings.
pub fn read_to_vec_string(path: &str) -> Vec<String> {
    let f = File::options()
        .read(true)
        .append(true)
        .create(true)
        .open(path)
        .expect("Error opening file! \n");

    let reader = BufReader::new(f);
    let mut v: Vec<String> = Vec::new();
    for line in reader.lines() {
        v.push(line.unwrap());
    }
    v
}

/// read file, and return values within a Vector of Strings.
// pub fn read_to_vec_u8<P: AsRef<Path>>(path: P) -> Vec<u8> {
pub fn read_to_vec_u8(path: &str) -> Vec<u8> {
    // let fp = util::path::get_full_file_path(path).unwrap();
    // dbg!(&path, &fp);
    let f = std::fs::read_to_string(path).expect("Can't open/read file!");
    f.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_to_vec_string() {
        let s = String::from("The Project Gutenberg eBook of Dracula");
        let dracula = "./dracula.txt";
        let res = read_to_vec_string(dracula);
        assert_eq!(s, res[0]);
    }
}
