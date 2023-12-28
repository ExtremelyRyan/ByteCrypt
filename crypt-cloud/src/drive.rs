use anyhow::{Error, Ok, Result};
use async_recursion::async_recursion;
pub use crypt_core::{
    common::DirInfo,
    common::{FileInfo, FsNode},
    token::UserToken,
};
use reqwest::{
    header::{CONTENT_LENGTH, CONTENT_RANGE, LOCATION},
    Client, Response,
};
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

const GOOGLE_FOLDER: &str = "Crypt";
pub const GOOGLE_CLIENT_ID: &str =
    "1006603075663-bi4o75nk6opljg7bicdiuden76s3v18f.apps.googleusercontent.com";
const CHUNK_SIZE: usize = 5_242_880; //5MB

/// <b>Asynchronously</b> sends an HTTP GET request to the specified URL with the provided user credentials.
///
/// # Arguments
///
/// * `url` - A string slice or reference to the URL to send the request to.
/// * `creds` - A reference to a `UserToken` containing the necessary credentials, including the access token.
/// # Errors
///
/// This function may return an error if the request fails. Possible error types include
/// network issues, authentication failures, or server errors.
///
/// # Panics
///
/// This function could panic if `reqwest` crate fails to create a new `Client`
pub async fn request_url(url: &str, creds: &UserToken) -> Result<Response, Error> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .bearer_auth(&creds.access_token)
        .send()
        .await
        .map_err(Error::from)?;
    Ok(response)
}

//Takes in an id and checks if that id exists on Google Drive
pub async fn g_id_exists(id: &str, creds: UserToken) -> Result<bool> {
    //Create the URL, we don't care about trashed items
    let url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?fields=trashed",
        id,
    );

    //Send the url and get the response
    let response = request_url(&url, &creds).await?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let files = response.json::<Value>().await?;
            return Ok(!files["trashed"].as_bool().unwrap_or(true));
        }
        reqwest::StatusCode::NOT_FOUND => return Ok(false),
        _ => {
            let error = response.json::<Value>().await?;
            return Err(Error::msg(format!(
                "Could not query Google Drive: {:?}",
                error
            )));
        }
    }
}

///Parse the drive and create the folder if it doesn't exist
pub async fn g_create_folder(
    creds: &UserToken,
    path: Option<&PathBuf>,
    parent: &str,
) -> Result<String> {
    let save_path = match path {
        Some(p) => p.to_str().unwrap(),
        None => GOOGLE_FOLDER,
    };

    //Check if the folder exists
    let query = format!(
        "name = '{}' and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
        save_path
    );
    let url = format!("https://www.googleapis.com/drive/v3/files?q={}", query);
    //Send the url and get the response
    let response = request_url(&url, &creds).await?;

    //If drive query failed, break out and print error
    if !response.status().is_success() {
        return Err(Error::msg(format!("{:?}", response.text().await?)));
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
            "parents": [parent]
        }),
        None => serde_json::json!({
            "name": save_path,
            "mimeType": "application/vnd.google-apps.folder",
        }),
    };
    //If folder doesn't exist, create new folder
    let _ = Client::new()
        .post("https://www.googleapis.com/drive/v3/files")
        .bearer_auth(&creds.access_token)
        .json(&json)
        .send()
        .await?;
    //Re-query to get folder id
    let query = format!(
        "name = '{}' and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
        save_path
    );
    let url = format!("https://www.googleapis.com/drive/v3/files?q={}", query);

    //Send the url and get the response
    let response = request_url(&url, &creds).await?;

    //If drive query failed, break out and print error
    if !response.status().is_success() {
        return Err(Error::msg(format!("{:?}", response.text().await?)));
    }
    //Search through and return id
    let folders = response.json::<Value>().await?;
    for item in folders["files"].as_array().unwrap_or(&vec![]) {
        if item["name"].as_str() == Some(save_path) {
            if let Some(id) = item["id"].as_str() {
                return Ok(id.to_string());
            }
        }
    }
    // println!("Error creating folder: {:?}", response.text().await?);
    return Err(Error::msg("Could not create folder".to_string()));
}

///Uploads a file to google drive
pub async fn g_upload(creds: UserToken, path: &str, parent: &str) -> Result<String> {
    //Get file content
    let mut file = tokio::fs::File::open(path).await?;
    let file_name = std::path::Path::new(path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let file_size = std::fs::metadata(path)?.len();

    let client = reqwest::Client::new();
    let response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable")
        .bearer_auth(&creds.access_token)
        .json(&serde_json::json!({
            "name": file_name,
            "parents": [parent]
        }))
        .header("X-Upload-Content-Type", "application/x-crypt") //application/octet-stream for unknown file types
        .send()
        .await?;

    let session_uri = response
        .headers()
        .get(LOCATION)
        .ok_or_else(|| Error::msg("Location header missing"))?
        .to_str()?
        .to_string();

    let mut start = 0;
    while start < file_size {
        let mut buffer = vec![0; CHUNK_SIZE];
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        buffer.truncate(bytes_read);

        let inner_response = client
            .put(&session_uri)
            .header(
                CONTENT_RANGE,
                format!(
                    "bytes {}-{}/{}",
                    start,
                    start + bytes_read as u64 - 1,
                    file_size
                ),
            )
            .header(CONTENT_LENGTH, bytes_read)
            .body(buffer[..bytes_read].to_vec())
            .send()
            .await?;

        match inner_response.status().as_u16() {
            308 => {
                //Incomplete continue
                //if log, place log here
            }
            200 | 201 => {
                let body = inner_response.json::<Value>().await?;
                if let Some(id) = body["id"].as_str() {
                    return Ok(id.to_string());
                } else {
                    return Err(Error::msg("Failed retrieve file ID"));
                }
            }
            //TODO: Deal with HTTP 401 Unauthorized Error
            status => {
                return Err(Error::msg(format!("Failed to upload: {}", status)));
            }
        }

        start += bytes_read as u64;
    }

    Err(Error::msg("File upload not successful"))
}

///Query google drive and return a Vec<String> of each item within the relevant folder
pub async fn g_view(name: &str, creds: UserToken) -> Result<Vec<String>> {
    //Get the folder id
    let mut folder_id = String::new();
    let query = format!(
        "name = '{}' and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
        name
    );
    let url = format!("https://www.googleapis.com/drive/v3/files?q={}", query);

    //Send the url and get the response
    let response = request_url(&url, &creds).await?;

    //If drive query failed, break out and print error
    if !response.status().is_success() {
        return Err(Error::msg(format!("{:?}", response.text().await?)));
    }
    //Search through response and return id
    let folders = response.json::<Value>().await?;
    for item in folders["files"].as_array().unwrap_or(&vec![]) {
        if item["name"].as_str() == Some(name) {
            if let Some(id) = item["id"].as_str() {
                folder_id = id.to_string();
            }
        }
    }
    //Use the ID to now get the folder's contents
    let url = format!(
        "https://www.googleapis.com/drive/v3/files?q='{}' in parents",
        folder_id
    );

    //Send the url and get the response
    let response = request_url(&url, &creds).await?;

    //If successful, convert to vec<string>
    if response.status().is_success() {
        let files = response.json::<Value>().await?;
        Ok(match files["files"].as_array() {
            Some(array) => array
                .iter()
                .filter_map(|item| item["name"].as_str())
                .map(String::from)
                .collect(),
            None => Vec::new(),
        })
    } else {
        Err(Error::msg("Could not query folder"))
    }
}

///Walks the google drive folder from a given folder name
pub async fn g_walk(name: &str, creds: UserToken) -> Result<DirInfo> {
    let client = reqwest::Client::new();
    //Get the folder id
    let query = format!(
        "name = '{}' and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
        name
    );
    let url = format!("https://www.googleapis.com/drive/v3/files?q={}", query);

    //Send the url and get the response
    let response = request_url(&url, &creds).await?;

    //If drive query failed, break out and print error
    if !response.status().is_success() {
        return Err(Error::msg(format!("{:?}", response.text().await?)));
    }
    //Search through response and return id
    let folders = response.json::<Value>().await?;
    for item in folders["files"].as_array().unwrap_or(&vec![]) {
        if item["name"].as_str() == Some(name) {
            if let Some(id) = item["id"].as_str() {
                return walk_cloud(&client, id, &creds).await;
            }
        }
    }

    return Err(Error::msg("Folder not found"));
}

/// Query google using file_id and download contents
///
/// TEMP: downloading
pub async fn google_query_file(file_id: &str, creds: UserToken) -> Result<()> {
    let url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?alt=media&source=downloadUrl",
        file_id
    );
    //Send the url and get the response
    let response = request_url(&url, &creds).await?;

    //If drive query failed, break out and print error
    if !response.status().is_success() {
        return Err(Error::msg(format!("{:?}", response.text().await?)));
    }

    let text = &response.bytes().await?;
    // TODO: Move somewhere else.
    // TODO: Also, get name from file and use that instead of "downloaded".
    _ = std::fs::write("downloaded.crypt", text);
    Ok(())
}

#[async_recursion]
async fn walk_cloud(
    client: &reqwest::Client,
    folder_id: &str,
    creds: &UserToken,
) -> Result<DirInfo> {
    let mut contents = Vec::new();
    let url = format!(
        "https://www.googleapis.com/drive/v3/files?q='{}' in parents and trashed = false",
        folder_id
    );
    //Send the url and get the response
    let response = request_url(&url, &creds).await?;

    if !response.status().is_success() {
        return Err(Error::msg("Could not view folder"));
    }

    let files = response.json::<Value>().await?;
    if let Some(array) = files["files"].as_array() {
        for item in array {
            let name = item["name"].as_str().unwrap_or_default().to_string();
            let id = item["id"].as_str().unwrap_or_default().to_string();

            if item["mimeType"] == "application/vnd.google-apps.folder" {
                let dir_info = walk_cloud(client, &id, creds).await?;
                contents.push(FsNode::Directory(dir_info));
            } else {
                contents.push(FsNode::File(FileInfo::new(name, id)));
            }
        }
    }

    let url = format!("https://www.googleapis.com/drive/v3/files/{}", folder_id);

    let dir_name = client
        .get(&url)
        .bearer_auth(&creds.access_token)
        .send()
        .await?
        .json::<Value>()
        .await?["name"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    Ok(DirInfo::new(
        dir_name,
        folder_id.to_string(),
        true,
        contents,
    ))
}

// --------------------------------------  UNUSED  --------------------------------------
///Gets drive info from google drive
pub async fn g_drive_info(creds: &UserToken) -> Result<Vec<Value>> {
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

        //Send the url and get the response
        let response = request_url(&url, &creds).await?;

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
