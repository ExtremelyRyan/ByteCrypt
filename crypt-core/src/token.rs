use crate::{
    common::{get_crypt_folder, send_information},
    config::get_config,
    db,
    encryption::{compress, decompress, generate_seeds},
    encryption::{KEY_SIZE, NONCE_SIZE},
};
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, Key, KeyInit, Nonce};
use lazy_static::lazy_static;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, CsrfToken, RedirectUrl, ResponseType, Scope};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    path::Path,
    time::{SystemTime, UNIX_EPOCH}
};
use url::form_urlencoded;

const _GOOGLE_FOLDER: &str = "Crypt";
pub const GOOGLE_CLIENT_ID: &str =
    "1006603075663-bi4o75nk6opljg7bicdiuden76s3v18f.apps.googleusercontent.com";
const _CHUNK_SIZE: usize = 1_048_576; //1MB

lazy_static! {
    ///Path for the google user token
    pub static ref GOOGLE_TOKEN_PATH: String = {
        let mut path = get_crypt_folder();
        path.push(".config");

        if !path.exists() {
            _ = std::fs::create_dir(&path);
        }
        path.push(".google");
        format!("{}", path.display())
    };

    ///Path for the dropbox user token
    pub static ref DROPBOX_TOKEN_PATH: String = {
        let mut path = get_crypt_folder();
        path.push(".config");

        if !path.exists() {
            _ = std::fs::create_dir(&path);
        }
        path.push(".dropbox");
        format!("{}", path.display())
    };
}

///Supported cloud platforms
///
/// # Options:
///```ignore
/// # use crypt_lib::util::CloudPlatform;
/// CloudPlatform::Google
/// CloudPlatform::DropBox
///```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CloudService {
    Google,
    Dropbox,
}

///For conversion to String from enum
impl ToString for CloudService {
    fn to_string(&self) -> String {
        match self {
            Self::Google => "Google".to_string(),
            Self::Dropbox => "DropBox".to_string(),
        }
    }
}

///For conversion from &str to enum
impl From<&str> for CloudService {
    fn from(service: &str) -> Self {
        match service {
            "Google" => Self::Google,
            "DropBox" => Self::Dropbox,
            _ => panic!("Invalid platform"),
        }
    }
}

///For conversion from String to enum
impl From<String> for CloudService {
    fn from(service: String) -> Self {
        match service.as_str() {
            "Google" => Self::Google,
            "DropBox" => Self::Dropbox,
            _ => panic!("Invalid platform"),
        }
    }
}

///Supported tasks for cloud platforms
///
/// # Options:  
/// ```ignore
/// * CloudTask::Upload
/// * CloudTask::Download
/// * CloudTask::View
/// ```
#[derive(Debug)]
pub enum CloudTask {
    Upload,
    Download,
    View,
}

///Holds user authentication information
///
/// # Fields
///``` ignore
/// UserToken {
///     service: CloudPlatform,
///     key_seed: [u8; 32],
///     nonce_seed: [u8; 12],
///     expiration: u64,
///     access_token: String,
/// }
///```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserToken {
    ///Platform the token is used for
    pub service: CloudService,
    ///Key for en/decrypting the user token
    pub key_seed: [u8; KEY_SIZE],
    ///Nonce for en/decrypting the user token
    pub nonce_seed: [u8; NONCE_SIZE],
    ///Time stamp for the user token
    pub expiration: u64,
    ///Grants access to the user account
    pub access_token: String,
}

impl UserToken {
    /// Generate a new user token to use with Google Drive.
    /// - Prompts user with link to authenticate with google.
    /// - Once the user successfully authenticates, a token will be created.
    ///
    /// # Options:
    ///```ignore
    /// let google_token = UserToken::new_google();
    ///```
    #[allow(clippy::manual_flatten)]
    pub fn new_google() -> Self {
        //Check if user_token already exists in database
        let user_token = get_access_token(CloudService::Google);
        if let Some(user_token) = user_token {
            return user_token;
        }

        // Set up the config for the Google OAuth2 process.
        let client = BasicClient::new(
            ClientId::new(GOOGLE_CLIENT_ID.to_string()),
            None, //No secret for implicit flow
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .expect("Invalid authorization endpoint URL"),
            None,
        )
        .set_redirect_uri(
            //Use a local server to redirect
            RedirectUrl::new("http://localhost:3000".to_string()).expect("Invalid redirect URL"),
        );

        //Authorization URL to redirect the user
        let (authorize_url, _) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/drive".to_string(),
            ))
            .use_implicit_flow()
            .set_response_type(&ResponseType::new("token".to_string()))
            .url();

        send_information(vec![format!(
            "Open this URL to authorize this application:\n{}\n",
            authorize_url
        )]);
        let mut token: Option<String> = None;
        let mut expires_in: Option<u64> = None;

        //Redirect server that grabs the token
        let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                // Read the HTTP request
                let mut reader = BufReader::new(&stream);
                let mut request = String::new();
                reader.read_line(&mut request).unwrap();

                // Check for GET request and serve the HTML with JavaScript
                if request.starts_with("GET") {
                    let html = r#"
                        <html>
                        <body>
                            <script>
                            window.onload = function() {
                                var hash = window.location.hash.substr(1);
                                var xhr = new XMLHttpRequest();
                                xhr.open("POST", "http://localhost:3000/token", true);
                                xhr.setRequestHeader("Content-Type", "application/x-www-form-urlencoded");
                                xhr.send(hash);
                            };
                            </script>
                            <p>You can now close this page and return to the applciation</p>
                        </body>
                        </html>
                    "#;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                        html.len(),
                        html
                    );
                    stream.write_all(response.as_bytes()).unwrap();
                }
                // Check for POST request to /token
                else if request.starts_with("POST /token") {
                    let mut content_length = 0;
                    let mut headers = String::new();

                    //read the line until breakpoint reached
                    while reader.read_line(&mut headers).unwrap() > 0 {
                        //Get the length of the body
                        if headers.starts_with("Content-Length:") {
                            content_length = headers
                                .split_whitespace()
                                .nth(1)
                                .unwrap()
                                .parse::<usize>()
                                .unwrap();
                        }
                        //break out of the loop if end reached
                        if headers == "\r\n" {
                            break;
                        }
                        headers.clear();
                    }
                    //Read the body
                    let mut body_buffer = vec![0_u8; content_length];
                    reader.read_exact(&mut body_buffer).unwrap();
                    let body = String::from_utf8(body_buffer).unwrap();

                    //Extract the token
                    let body_parts: HashMap<_, _> = form_urlencoded::parse(body.as_bytes())
                        .into_owned()
                        .collect();
                    token = body_parts.get("access_token").cloned();
                    expires_in = body_parts
                        .get("expires_in")
                        .and_then(|v| v.parse::<u64>().ok());

                    //Respond to close connection
                    let response = "HTTP/1.1 200 OK\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                    break; //shut down server
                }
            }
        }
        let token = match token {
            Some(token) => token,
            None => {
                send_information(vec![format!("Unable to get access token")]);
                "".to_string()
            }
        };
        let expires_in = match expires_in {
            Some(expires_in) => expires_in,
            None => {
                send_information(vec![format!("Unable to get token expiration information")]);
                0
            }
        };
        //Create the user_token
        let (key_seed, nonce_seed) = generate_seeds();
        let user_token = Self {
            service: CloudService::Google,
            key_seed,
            nonce_seed,
            expiration: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Somehow, time has gone backwards")
                .as_secs()
                + expires_in,
            access_token: token.to_string(),
        };

        let _ = db::insert_token(&user_token);
        let _ = save_access_token(&user_token);
        user_token
    }

    /// Generate a new user token to use with Dropbox.
    /// - Prompts user with link to authenticate with Dropbox.
    /// - Once the user successfully authenticates, a token will be created.
    ///
    /// # Options:
    ///```ignore
    /// let dropbox_token = UserToken::new_dropbox();
    ///```
    pub fn new_dropbox() -> Self {
        let client_id = "im68gew9aehy2pn".to_string();

        let client = BasicClient::new(
            ClientId::new(client_id),
            None,
            AuthUrl::new("https://www.dropbox.com/oauth2/authorize".to_string())
                .expect("Invalid authorization endpoint URL"),
            None,
        )
        .set_redirect_uri(RedirectUrl::new("http://localhost:3000".to_string()).unwrap());

        let (_authorize_url, _csrf_state) = client.authorize_url(CsrfToken::new_random).url();

        todo!()
    }
}

///Attempts to get an access token from the database
fn get_access_token(service: CloudService) -> Option<UserToken> {
    //Get the path
    let path = match service {
        CloudService::Google => GOOGLE_TOKEN_PATH.as_str(),
        CloudService::Dropbox => DROPBOX_TOKEN_PATH.as_str(),
    };
    //Test if the path exists
    if !Path::new(path).exists() {
        return None;
    }
    //Read the token from the location
    let access_token = fs::read(path).unwrap();

    //Ensure that it's not expired
    match db::query_token(service) {
        Ok(mut user_token) => {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Somehow, time has gone backwards")
                .as_secs();

            match user_token.expiration > current_time {
                true => {
                    user_token.access_token = decrypt_token(&user_token, access_token);
                    Some(user_token)
                }
                false => None,
            }
        }
        Err(_) => None,
    }
}

fn save_access_token(user_token: &UserToken) -> anyhow::Result<()> {
    //Get the path
    let path = match user_token.service {
        CloudService::Google => GOOGLE_TOKEN_PATH.as_str(),
        CloudService::Dropbox => DROPBOX_TOKEN_PATH.as_str(),
    };
    let token = encrypt_token(user_token)?;

    fs::write(path, token)?;

    Ok(())
}

pub fn encrypt_token(user_token: &UserToken) -> anyhow::Result<Vec<u8>> {
    let conf = get_config();
    let mut token = user_token.access_token.as_bytes();
    let compressed_token = compress(token, conf.zstd_level);
    token = compressed_token.as_slice();

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&user_token.key_seed))
        .encrypt(Nonce::from_slice(&user_token.nonce_seed), token)
        .expect("Failed to encrypt access_token");
    Ok(cipher)
}

pub fn decrypt_token(user_token: &UserToken, access_token: Vec<u8>) -> String {
    let token = access_token.as_slice();

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&user_token.key_seed))
        .decrypt(Nonce::from_slice(&user_token.nonce_seed), token.as_ref())
        .expect("Failed to decrypt access_token");

    let decompressed_token = decompress(cipher.as_slice());

    String::from_utf8(decompressed_token).expect("Could not decrypt token")
}

pub fn purge_tokens() {
    let mut path = get_crypt_folder();
    path.push(".config");

    path.push(".google"); 
    if path.exists() {
        _ = std::fs::remove_file(&path);
        send_information(vec![format!("removed google token file.")]);
    }
    path.pop();
    path.push(".dropbox"); 
    if path.exists() {
        _ = std::fs::remove_file(&path);
        send_information(vec![format!("removed dropbox token file.")]);
    }
}