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

pub fn _write_to_file(path: &str, file_crypt: FileCrypt) -> anyhow::Result<()> {
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
