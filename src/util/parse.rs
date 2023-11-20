use super::encryption::FileCrypt; 
use std::{fs::File, io::Read, io::Write};
use toml;

pub fn toml_example() -> anyhow::Result<FileCrypt> {
    Ok(toml::from_str(
        r#"
        filename = "foo.txt"
        full_path = "C:/Users/ryanm/code/bytecloak/foo.txt"
 
        key = [1,2,3,4,5]
        nonce = [6,99,7,6,6,87,5,4,6,6]
        "#,
    )
    .expect("error Serializing"))
}

pub fn write_to_file(path: &str, file_crypt: FileCrypt) -> anyhow::Result<()> {
    let mut f = File::create(path)?;
    let buf = toml::to_string(&file_crypt).unwrap();
    let bytes = buf.as_bytes();
    f.write_all(bytes[..].as_ref())?;
    Ok(())
}

/// simple prepending file
pub fn prepend_file(file_crypt: FileCrypt, path: &str) -> anyhow::Result<()> {
    // open file
    let mut f = File::open(path)?;
    // insert new data into vec
    let mut content = toml::to_string(&file_crypt)
        .expect("error serializing data!")
        .as_bytes()
        .to_owned();
    content.push(b'\n');
    f.read_to_end(&mut content).expect("error reading file");

    let mut f = File::create(path).expect("error creating file");
    f.write_all(content.as_slice())
        .expect("error writing to file");

    Ok(())
}


pub fn remove_data(file_crypt: FileCrypt, path: &str) -> anyhow::Result<()> {
    // open file
    let mut f = File::options().append(true).open(path)?;

    let mut s: String = String::new();
    std::io::BufReader::new(&f).read_to_string(&mut s)?;

    let mut crypt: Vec<String> = s.split('\n')
        // .filter(|s| !s.is_empty()) // so long as the string is not empty
        .map(|s| s.to_string()) // convert item to a string.
        .collect();

    dbg!(&file_crypt.filename);
    let index = crypt.iter().position(|r| r.contains(&file_crypt.filename)).expect("cant find filename in crypt file!");
    dbg!(&index);

    crypt.drain(index..index+4); 
    dbg!(&crypt);

    let temp: &[String] = crypt.iter().as_slice();
  
    let mut bytes: Vec<u8> = Vec::new();

    for t in temp { 
        for tt in t.as_bytes() {
            bytes.push(*tt);
        }
    }
    let res = f.write_all(bytes.as_slice());
    dbg!(&res);
    Ok(())
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