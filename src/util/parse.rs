use super::encryption::FileCrypt;
use anyhow::{Result,Ok};
use std::{
    fs::{File, OpenOptions},
    io::Read,
    io::Write,
};


pub fn write_contents_to_file(file: &str, encrypted_contents: Vec<u8>) -> Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(file)?;
    f.write_all(encrypted_contents.as_slice())
        .expect("failed writing to file");
    Ok(f.flush()?)
}

pub fn prepend_uuid(uuid: &String, encrypted_contents: &mut Vec<u8>) -> Vec<u8> { 
    let mut uuid_bytes = uuid.as_bytes().to_vec();
    let mut tmp: Vec<u8> = vec![33,33,33];
    uuid_bytes.append(&mut tmp);
    uuid_bytes.append(encrypted_contents);

    println!("== parse.rs: printing uuid_bytes");
    print!("    ");
    for i in 0..39 {
        print!("{:?}",uuid_bytes.get(i).unwrap());
    }
    println!("\n");   
    uuid_bytes
}

pub trait RemoveElem<T> {
    fn remove_elem<F>(&mut self, predicate: F) -> Option<T>
    where
        F: Fn(&T) -> bool;
}

impl<T> RemoveElem<T> for Vec<T> {
    fn remove_elem<F>(&mut self, predicate: F) -> Option<T>
    where
        F: Fn(&T) -> bool,
    {
        self.iter()
            .position(predicate)
            .map(|index| self.remove(index))
    }
}
