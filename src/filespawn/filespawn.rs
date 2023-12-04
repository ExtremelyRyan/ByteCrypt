use rand::{Rng, distributions::Alphanumeric};
use std::{
    path::{Path, PathBuf},
    fs::{ File, create_dir_all}
}; 
use std::io::Write;

const SAVE_PATH: &str = "src/filespawn/test_files/";
const NUM_FILES: u16 = 1000;

#[derive(Debug)]
struct RFile {
    name: String,
    content: Vec<String>,
}

pub fn generate_files() -> anyhow::Result<()> {
    //If the directory doesn't exist, create it
    if !Path::new(SAVE_PATH).exists() {
        println!("Test directory does not exist, generating new directory...");
        let path = PathBuf::from(SAVE_PATH);
        _ = create_dir_all(path)?;
    }

    
    let mut files: Vec<RFile> = Vec::new();
    for i in 0..NUM_FILES {
        let file = generate_random_file(i.to_string());
        let file_path = format!("{}{}", SAVE_PATH, file.name);
        let mut out = File::create(&file_path)?;

        for line in &file.content {
            writeln!(out, "{}", line)?;
        }
        files.push(file);
    }

    return Ok(());
}

fn generate_random_file(name: String) -> RFile {
    let mut rng = rand::thread_rng();
    let content_height: usize = rng.gen_range(1..1000);
    let mut strings: Vec<String> = Vec::new(); 
    for _ in 1..=content_height {
        let content_width = rng.gen_range(10..1000);
        let random_content: String = (0..content_width)
                                        .map(|_| rng.sample(Alphanumeric) as char)
                                        .collect();
        strings.push(random_content);
    }
    
    let output = RFile {
        name: format!("{}{}", name, ".txt"),
        content: strings
    };

    output
}
