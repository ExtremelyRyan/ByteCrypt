use super::oauth::UserToken;
use std::path::PathBuf;
use http::Response;
use reqwest::header::{CONTENT_LENGTH, CONTENT_RANGE, LOCATION};
use serde_json::{from_reader, Value};
use tokio::io::AsyncReadExt;

const GOOGLE_FOLDER: &str = "Crypt";
pub const GOOGLE_CLIENT_ID: &str = "1006603075663-bi4o75nk6opljg7bicdiuden76s3v18f.apps.googleusercontent.com";
const CHUNK_SIZE: usize = 1_048_576; //1MB

///Gets drive info from google drive
pub async fn g_drive_info(creds: &UserToken) -> anyhow::Result<Vec<Value>> {
    //Token to query the drive
    let mut page_token: Option<String> = None;
    let mut values: Vec<Value> = Vec::new();
    //Loop through each page
    loop {
        let url = match &page_token {
            Some(token) => {
                format!(
                    "https://www.googleapis.com/drive/v3/files?pageToken={}",
                    token
                )
            }
            None => "https://www.googleapis.com/drive/v3/files".to_string(),
        };

        let response = reqwest::Client::new()
            .get(url)
            .bearer_auth(&creds.access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let stuff = response.json::<Value>().await?;
            println!("{:#?}", &stuff);
            values.push(stuff.clone());

            if let Some(next_token) = stuff["nextPageToken"].as_str() {
                page_token = Some(next_token.to_string());
            } else {
                break;
            }
        } else {
            println!("Error {:?}", response.text().await?);
            break;
        }
    }
    Ok(values)
}

///Parse the drive and create the folder if it doesn't exist
pub async fn g_create_folder(creds: &UserToken, path: Option<&PathBuf>) -> anyhow::Result<String> {
    let save_path = match path {
        Some(p) => p.to_str().unwrap(),
        None => GOOGLE_FOLDER,
    };
    let client = reqwest::Client::new();

    //Check if the folder exists
    let query = format!(
        "name = '{}' and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
        save_path
    );
    let url = format!("https://www.googleapis.com/drive/v3/files?q={}", query);
    let response = client
        .get(url)
        .bearer_auth(&creds.access_token)
        .send()
        .await?;
    //If drive query failed, break out and print error
    if !response.status().is_success() {
        return Err(anyhow::Error::msg(format!("{:?}", response.text().await?)));
    }
    //If folder exists, break out
    let folders = response.json::<Value>().await?;
    for item in folders["files"].as_array().unwrap_or(&vec![]) {
        if item["name"].as_str() == Some(save_path) {
            if let Some(id) = item["id"].as_str() {
                return Ok(id.to_string());
            }
        }
    }
    //Make sure the folder is created within the crypt folder
    let json = match path {
        Some(_) => serde_json::json!({
            "name": save_path,
            "mimeType": "application/vnd.google-apps.folder",
            "parents": [GOOGLE_FOLDER] 
        }),
        None => serde_json::json!({
            "name": save_path,
            "mimeType": "application/vnd.google-apps.folder",
        }),
    };
    //If folder doesn't exist, create new folder
    let inner_response = client
            .post("https://www.googleapis.com/drive/v3/files")
            .bearer_auth(&creds.access_token)
            .json(&json)
            .send()
            .await?;
    //Query the list of folders to get folder id
    let folders = inner_response.json::<Value>().await?;
    for item in folders["files"].as_array().unwrap_or(&vec![]) {
        if item["name"].as_str() == Some(save_path) {
            if let Some(id) = item["id"].as_str() {
                return Ok(id.to_string());
            }
        }
    } 
    // println!("Error creating folder: {:?}", response.text().await?);
    return Err(anyhow::Error::msg("Could not create folder".to_string()));
}

///Uploads a file to google drive
pub async fn g_upload(creds: UserToken, path: &str, folder_id: String) -> anyhow::Result<()> {
    //Get file content
    let mut file = tokio::fs::File::open(path).await?;
    let file_name = std::path::Path::new(path).file_name().unwrap().to_str().unwrap();
    let file_size = std::fs::metadata(path)?.len();
    
    let client = reqwest::Client::new();
    let response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable")
        .bearer_auth(&creds.access_token)
        .json(&serde_json::json!({
            "name": file_name,
            "parents": [folder_id]
        }))
        .header("X-Upload-Content-Type", "application/x-crypt") //application/octet-stream for unknown file types
        .send()
        .await?;

    let session_uri = response.headers().get(LOCATION)
        .ok_or_else(|| anyhow::Error::msg("Location header missing"))?
        .to_str()?.to_string();

    let mut start = 0;
    while start < file_size {
        let mut buffer = vec![0; CHUNK_SIZE];
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 { break; }

        buffer.truncate(bytes_read);

        let inner_response = client
            .put(&session_uri)
            .header(
                CONTENT_RANGE, 
                format!("bytes {}-{}/{}", start, start + bytes_read as u64 - 1, file_size)) 
            .header(CONTENT_LENGTH, bytes_read)
            .body(buffer[..bytes_read].to_vec())
            .send()
            .await?;

        match inner_response.status().as_u16() {
            308 => {
                //Incomplete continue
                //if log, place log here
            },
            200 | 201 => {
                println!("we did it bois");
                break;
            },
            status => {
                return Err(anyhow::Error::msg(format!("Failed to upload: {}", status)));
            },
        }

        start += bytes_read as u64;
    }

    Ok(())
}

//Check if drive path exists
// - Name, MIME type, etc.

//Create folder if none
// - MIME type application/vnd.google-apps.folder
// - Create any subsequent folders

//Upload file
