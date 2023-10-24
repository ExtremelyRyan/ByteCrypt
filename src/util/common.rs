use std::{fs::{File}, path::{Path}, io::{BufReader, BufRead}};

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
        // let mut l = line.unwrap();  
        // let offset = l.find('C').unwrap_or(7);
        // l.replace_range(0..offset, ""); 
        v.push(line.unwrap());
    }
    v
    // read_to_string(f.)
    //     .expect("Can't open/read file!")
    //     .split("\n")
    //     .filter(|s| !s.is_empty()) // so long as the string is not empty
    //     .map(|s| s.trim().to_string()) // convert item to a string.
    //     .collect()
}

/// read file, and return values within a Vector of Strings.
pub fn read_to_vec_u8<T: AsRef<Path>>(path: T) -> Vec<u8> {
    std::fs::read(path).expect("Can't open/read file!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_to_vec_string() {
        let s = String::from("The Project Gutenberg eBook of Dracula");
        let dracula = "./test.txt";
        let res = read_to_vec_string(dracula);
        assert_eq!(s, res[0]);
    }
}
