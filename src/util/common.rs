use std::{fs::read_to_string, path::Path};

/// read file, and return values within a Vector of Strings.
pub fn read_to_vec_string<T: AsRef<Path>>(path: T) -> Vec<String> {
    read_to_string(path)
        .expect("Can't open/read file!")
        .split("\n")
        .filter(|s| !s.is_empty()) // so long as the string is not empty
        .map(|s| s.trim().to_string()) // convert item to a string.
        .collect()
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
        let dracula = String::from("./test.txt");
        let res = read_to_vec_string(dracula);
        assert_eq!(s, res[0]);
    }
}
