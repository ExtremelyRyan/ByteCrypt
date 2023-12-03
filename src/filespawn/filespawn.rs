use rand::{Rng, distributions::Alphanumeric};
use std::fs::File;
use std::io::Write;

const SAVE_PATH: &str = "test_files";

#[derive(Debug)]
pub struct RFile {
    pub name: String,
    pub content: Vec<String>,
}

pub fn generate_files(amount: u32) {
    let mut files: Vec<RFile> = Vec::new();
    for i in 0..amount {
        files.push(generate_random_file(i.to_string()));
    }

    for file in files.iter() {
        let mut out = File::create(&file.name);

        for line in &file.content {
            writeln!(out, "{}", line);
        }
    }
}

pub fn generate_random_file(name: String) -> RFile {
    let mut rng = rand::thread_rng();
    let content_height: usize = rng.gen_range(1..1000);
    let mut strings: Vec<String> = Vec::new(); 
    for i in 1..=content_height {
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
