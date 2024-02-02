// Split the functionality into smaller functions

// Function to get file information
fn get_file_info(path: &str) -> (String, String, String, String) {
    let (fp, _, filename, extension) = get_file_info(path);
    (fp, filename, extension)
}

// Function to create a FileCrypt instance
fn create_file_crypt(filename: String, extension: String, fp: String, contents: &[u8]) -> FileCrypt {
    FileCrypt::new(filename, extension, "".to_string(), fp, compute_hash(contents))
}

// Function to compress contents
fn compress_contents(contents: &[u8], zstd_level: i32) -> Vec<u8> {
    compress(contents, zstd_level).as_slice().to_vec()
}

// Function to encrypt contents
fn encrypt_contents(fc: &FileCrypt, contents: &[u8]) -> Vec<u8> {
    encrypt(fc, contents).unwrap()
}

// Function to prepend UUID to contents
fn prepend_uuid_to_contents(uuid: &Uuid, encrypted_contents: &mut Vec<u8>) {
    *encrypted_contents = prepend_uuid(uuid, encrypted_contents);
}

// Function to handle output path
fn handle_output_path(path: &str, output: &Option<String>, fc: &FileCrypt) -> PathBuf {
    let mut path = get_crypt_folder();
    match output {
        Some(o) => {
            let alt_path = get_alternative_path(&path, &o);
            create_directory_if_not_exists(&alt_path);
            path.push(format!(r#"{}\{}{}"#, o, fc.filename, ".crypt"));
        }
        None => path.push(format!("{}{}", fc.filename, ".crypt")),
    }
    path
}

// Function to insert FileCrypt data into the database
fn insert_crypt_into_database(fc: &FileCrypt) {
    insert_crypt(fc).expect("failed to insert FileCrypt data into database!");
}

// Function to write contents to file
fn write_contents_to_file(path: &str, encrypted_contents: Vec<u8>) {
    write_contents_to_file(path, encrypted_contents)
        .expect("failed to write contents to file!");
}

// Main function to encrypt a file
pub fn encrypt_file(path: &str, output: &Option<String>) {
    let conf = get_config();
    let (fp, filename, extension) = get_file_info(path);

    let contents = get_file_bytes(path);
    let fc = create_file_crypt(filename.clone(), extension.clone(), fp.clone(), &contents);

    let compressed_contents = compress_contents(&contents, conf.zstd_level);
    let mut encrypted_contents = encrypt_contents(&fc, &compressed_contents);

    prepend_uuid_to_contents(&fc.uuid, &mut encrypted_contents);

    let path = handle_output_path(path, output, &fc);

    insert_crypt_into_database(&fc);

    write_contents_to_file(path.to_str().unwrap(), encrypted_contents.clone());
}

//////////////////////////////////////////////////////////////////////////////

 
















