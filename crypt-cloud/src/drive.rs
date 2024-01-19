use anyhow::{Error, Ok, Result};
use async_recursion::async_recursion;
use crypt_core::{
    common::DirInfo,
    common::{FileInfo, FsNode},
    db::{insert_crypt, query_crypt},
    filecrypt::{encrypt_contents, get_uuid},
    token::UserToken,
};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_LENGTH, CONTENT_RANGE, LOCATION},
    Client, Response,
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::{fs::File, io::AsyncReadExt};

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
pub async fn g_id_exists(user_token: &UserToken, id: &str) -> Result<bool> {
    //Create the URL, we don't care about trashed items
    let url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?fields=trashed",
        id,
    );

    //Send the url and get the response
    let response = request_url(&url, user_token).await?;

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
    user_token: &UserToken,
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
    let response = request_url(&url, user_token).await?;

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
        .bearer_auth(&user_token.access_token)
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
    let response = request_url(&url, user_token).await?;

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

///Updates a file that already exists on google drive
pub async fn g_update(user_token: &UserToken, id: &str, path: &str) -> Result<String> {
    //Get file content
    let mut file = tokio::fs::File::open(path).await?;
    let file_size = std::fs::metadata(path)?.len();

    let client = reqwest::Client::new();
    let url = format!(
        "https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=resumable",
        id
    );

    let response = client
        .patch(&url)
        .bearer_auth(&user_token.access_token)
        .header("X-Upload-Content-Type", "application/x-crypt")
        .send()
        .await?
        .error_for_status()?;

    let session_uri = response
        .headers()
        .get(LOCATION)
        .ok_or_else(|| anyhow::Error::msg("Location header missing"))?
        .to_str()?
        .to_owned();

    return Ok(upload_chunks(&session_uri, &mut file, file_size).await?);
}

///Uploads a file to google drive
pub async fn g_upload(
    user_token: &UserToken,
    path: &str,
    parent: &str,
    no_encrypt: &bool,
) -> Result<String> {
    //Get file content
    // dbg!(&no_encrypt);
    let mut file = File::open(path).await?;
    let mut tmp; // to appease the compiler gods
    let mut file_name = Path::new(path).file_name().unwrap().to_str().unwrap();

    // if we are encrypting the file, get just the filename and append ".crypt"
    if !no_encrypt {
        file_name = Path::new(path).file_stem().unwrap().to_str().unwrap();
        tmp = String::from(file_name);
        tmp.push_str(".crypt");
        file_name = &tmp;
    }

    let file_size = std::fs::metadata(path)?.len();

    let client = reqwest::Client::new();
    let response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable")
        .bearer_auth(&user_token.access_token)
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

    // if the `no_encrypt` flag IS NOT true, assume we are encrypting contents to send.
    if !no_encrypt {
        let encrypted_content = encrypt_contents(path);
        if encrypted_content.is_some() {
            let ec = encrypted_content.unwrap();
            let drive_id = upload_content_chunks(&session_uri, &ec).await?;

            let (uuid, _) = get_uuid(&ec).unwrap();
            let mut filecrypt = query_crypt(uuid).unwrap();
            filecrypt.drive_id = drive_id.clone();
            _ = insert_crypt(&filecrypt);
            return Ok(drive_id);
        }
    }

    return Ok(upload_chunks(&session_uri, &mut file, file_size).await?);
}

///Helper function that performs the upload of file information
async fn upload_chunks(session_uri: &str, file: &mut File, file_size: u64) -> Result<String> {
    let client = reqwest::Client::new();

    let mut start = 0;
    while start < file_size {
        let mut buffer = vec![0; CHUNK_SIZE];
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        buffer.truncate(bytes_read);

        let inner_response = client
            .put(session_uri)
            .headers(build_headers(start, (bytes_read - 1) as u64, file_size))
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

    return Err(anyhow::Error::msg("File upload not successful"));
}

async fn upload_content_chunks(session_uri: &str, data: &[u8]) -> Result<String> {
    let client = reqwest::Client::new();
    let data_size = data.len() as u64;

    let mut start = 0;
    while start < data_size {
        let end = (start + CHUNK_SIZE as u64).min(data_size);
        let chunk = &data[start as usize..end as usize];

        let inner_response = client
            .put(session_uri)
            .headers(build_headers(start, end - 1, data_size))
            .body(chunk.to_vec())
            .send()
            .await?;

        match inner_response.status().as_u16() {
            308 => {
                // Incomplete continue
                // If log, place log here
            }
            200 | 201 => {
                let body = inner_response.json::<serde_json::Value>().await?;
                if let Some(id) = body["id"].as_str() {
                    return Ok(id.to_string());
                } else {
                    return Err(Error::msg("Failed to retrieve file ID"));
                }
            }
            // TODO: Deal with HTTP 401 Unauthorized Error
            status => {
                return Err(Error::msg(format!("Failed to upload: {}", status)));
            }
        }

        start = end;
    }

    Err(anyhow::Error::msg("File upload not successful"))
}

/// Builds and returns a set of HTTP headers for a partial content range request.
///
/// The function constructs the `Content-Range` and `Content-Length` headers based on the
/// specified start, end, and total size of the content range.
///
/// # Arguments
///
/// * `start` - The starting byte index of the content range.
/// * `end` - The ending byte index (inclusive) of the content range.
/// * `total_size` - The total size of the entire content.
///
/// # Returns
///
/// A `HeaderMap` containing the constructed HTTP headers.
///
/// # Panics
///
/// This function may panic if overflow occurs during the calculation of the `Content-Length`.
fn build_headers(start: u64, end: u64, total_size: u64) -> HeaderMap {
    let content_range = format!("bytes {}-{}/{}", start, end, total_size);

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_RANGE,
        HeaderValue::from_str(&content_range).unwrap(),
    );
    dbg!(&end, &start, &total_size);
    // Ensure no overflow by adding first and then subtracting
    let content_length = end.checked_add(1).and_then(|e| e.checked_sub(start));
    dbg!(&content_length);
    if let Some(length) = content_length {
        headers.insert(CONTENT_LENGTH, HeaderValue::from(length));
    } else {
        // TODO: find better way to handle this
        panic!("Overflow when calculating content length");
    }
    headers
}

///Query google drive and return a Vec<String> of each item within the relevant folder
pub async fn g_view(user_token: &UserToken, name: &str) -> Result<Vec<String>> {
    //Get the folder id
    let mut folder_id = String::new();
    let query = format!(
        "name = '{}' and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
        name
    );
    let url = format!("https://www.googleapis.com/drive/v3/files?q={}", query);

    //Send the url and get the response
    let response = request_url(&url, user_token).await?;

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
    let response = request_url(&url, user_token).await?;

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
pub async fn g_walk(user_token: &UserToken, name: &str) -> Result<DirInfo> {
    let client = reqwest::Client::new();
    //Get the folder id
    let query = format!(
        "name = '{}' and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
        name
    );
    let url = format!("https://www.googleapis.com/drive/v3/files?q={}", query);

    //Send the url and get the response
    let response = request_url(&url, user_token).await?;

    //If drive query failed, break out and print error
    if !response.status().is_success() {
        return Err(Error::msg(format!("{:?}", response.text().await?)));
    }
    //Search through response and return id
    let folders = response.json::<Value>().await?;
    for item in folders["files"].as_array().unwrap_or(&vec![]) {
        if item["name"].as_str() == Some(name) {
            if let Some(id) = item["id"].as_str() {
                return walk_cloud(user_token, &client, id).await;
            }
        }
    }

    return Err(Error::msg("Folder not found"));
}

/// Query google using file_id and download contents
///
/// TEMP: downloading
pub async fn google_query_file(user_token: &UserToken, file_id: &str) -> Result<Vec<u8>> {
    let url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?alt=media&source=downloadUrl",
        file_id
    );
    //Send the url and get the response
    let response = request_url(&url, &user_token).await?;

    //If drive query failed, break out and print error
    if !response.status().is_success() {
        return Err(Error::msg(format!("{:?}", response.text().await?)));
    }

    let bytes = &response.bytes().await?;
    let text = bytes.to_vec();
    // TODO: Move somewhere else.
    // TODO: Also, get name from file and use that instead of "downloaded".
    // _ = std::fs::write("downloaded.crypt", text);
    Ok(text)
}

///Walks google drive to get all of the files within their respective folders
#[async_recursion]
async fn walk_cloud(
    user_token: &UserToken,
    client: &reqwest::Client,
    folder_id: &str,
) -> Result<DirInfo> {
    let mut contents = Vec::new();
    let url = format!(
        "https://www.googleapis.com/drive/v3/files?q='{}' in parents and trashed = false",
        folder_id
    );
    //Send the url and get the response
    let response = request_url(&url, user_token).await?;

    if !response.status().is_success() {
        return Err(Error::msg("Could not view folder"));
    }

    let files = response.json::<Value>().await?;
    if let Some(array) = files["files"].as_array() {
        for item in array {
            let name = item["name"].as_str().unwrap_or_default().to_string();
            let id = item["id"].as_str().unwrap_or_default().to_string();

            if item["mimeType"] == "application/vnd.google-apps.folder" {
                let dir_info = walk_cloud(user_token, client, &id).await?;
                contents.push(FsNode::Directory(dir_info));
            } else {
                contents.push(FsNode::File(FileInfo::new(name, id)));
            }
        }
    }

    let url = format!("https://www.googleapis.com/drive/v3/files/{}", folder_id);

    let dir_name = client
        .get(&url)
        .bearer_auth(&user_token.access_token)
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
pub async fn g_drive_info(user_token: &UserToken) -> Result<Vec<Value>> {
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
        let response = request_url(&url, user_token).await?;

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
