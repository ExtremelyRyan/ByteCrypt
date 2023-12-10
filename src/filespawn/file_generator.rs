use rand::{distributions::Alphanumeric, Rng};
use std::io::Write;
use std::{
    collections::HashSet,
    fs::{create_dir_all, read_dir, File},
    path::Path,
};

pub const SAVE_PATH: &str = "benches/test_files/";
const NUM_FILES: u16 = 100;
const MAX_HEIGHT: usize = 1000;
const MIN_WIDTH: usize = 10;
const MAX_WIDTH: usize = 1000;

#[derive(Debug)]
struct RFile {
    name: String,
    content: Vec<String>,
}

///Generates a directory filled with randomly generated files
pub fn generate_files() -> anyhow::Result<()> {
    //If the directory doesn't exist, create it
    if !Path::new(SAVE_PATH).exists() {
        println!("Test directory does not exist, generating new directory...");
        create_dir_all(SAVE_PATH)?;
    }

    println!("Detecting files in directory");
    //Check the files in the directory
    let existing_files: HashSet<String> = read_dir(SAVE_PATH)?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    println!(
        "{} files detected. Checking and resolving any missing files.",
        existing_files.len()
    );

    let mut completed = 0;
    //Fill the folder
    for i in 0..NUM_FILES {
        let percentage = (completed as f64 / NUM_FILES as f64) * 100.0;
        let bar_length = 20;
        let filled_length = ((bar_length as f64 * completed as f64) / NUM_FILES as f64) as usize;
        let bar: String = "=".repeat(filled_length) + &" ".repeat(bar_length - filled_length);
        print!("\r[{:<20}] {:.0}%", bar, percentage);
        std::io::stdout().flush().unwrap();

        let file_name = format!("{}.txt", i);

        //Skip the file creation if it already exists
        if existing_files.contains(&file_name) {
            completed += 1;
            continue;
        }
        // println!("Generating random file: {}", file_name);
        //Generate random file and save it
        let file = generate_random_file(file_name);
        let file_path = format!("{}{}", SAVE_PATH, file.name);
        let mut out = File::create(&file_path)?;

        for line in &file.content {
            writeln!(out, "{}", line)?;
        }
        completed += 1;
    }

    return Ok(());
}

fn generate_random_file(name: String) -> RFile {
    let mut rng = rand::thread_rng();
    let content_height: usize = rng.gen_range(1..MAX_HEIGHT);
    let mut strings: Vec<String> = Vec::new();
    for _ in 1..=content_height {
        let content_width = rng.gen_range(MIN_WIDTH..MAX_WIDTH);
        let random_content: String = (0..content_width)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect();
        strings.push(random_content);
    }

    let output = RFile {
        name: name.to_string(),
        content: strings,
    };

    return output;
}
