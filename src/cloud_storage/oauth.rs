use oauth2::{
    basic::BasicClient, 
    reqwest::http_client,
    //reqwest::async_http_client,
    ResponseType, RevocationUrl, PkceCodeChallenge, RedirectUrl, CsrfToken,
    AuthType, AuthUrl, ClientId, ClientSecret, DeviceAuthorizationUrl,
    Scope, TokenUrl, DeviceCode, AuthorizationCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use url::Url;
use std::env;
use std::net::TcpListener;
use std::io::{Read, Write, BufReader, BufRead};
use std::str;



///Holds the user credentials for the session
#[derive(Deserialize)]
pub struct UserCredentials {
    ///Grants access to the user account
    pub access_token: String,
//    ///Type of token
    // token_type: String,
    // ///(Optional) -> get new access token when current one expires
    // refresh_token: String,
    // ///(Optional) -> Lifetime of token in seconds
    // expires_in: Option<u64>,
    // ///Scope of access and permissions granted
    // scope: Option<String>,
}

///Authenticate user with google and get access token for drive
pub fn google_access() -> anyhow::Result<()> {
    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        ClientId::new(
            env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
        ),
        None,//No secret for implicit flow
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .expect("Invalid authorization endpoint URL"),
        None,
    )
    .set_redirect_uri( //Use a local server to redirect
        RedirectUrl::new("http://localhost:3000".to_string()).expect("Invalid redirect URL"),
    );

    //Authorization URL to redirect the user
    let (authorize_url, _) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/drive.file".to_string(),
        ))
        .use_implicit_flow()
        .set_response_type(&ResponseType::new("token".to_string()))
        .url();

    println!(
        "Open this URL to authorize this application:\n{}\n",
        authorize_url.to_string()
    );

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
                        <p>Please wait...</p>
                    </body>
                    </html>
                "#;
                let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", html.len(), html);
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
                            .nth(1).unwrap()
                            .parse::<usize>()
                            .unwrap();
                    }
                    //break out of the loop if end reached
                    if headers == "\r\n" { break; }
                    headers.clear();
                }
                //Read the body
                let mut body_buffer = vec![0_u8; content_length];
                reader.read_exact(&mut body_buffer).unwrap();
                let mut body = String::from_utf8(body_buffer).unwrap();

                //Extract the token
                let token = body.split("&")
                    .find(|param| param.starts_with("access_token"))
                    .and_then(|param| param.split('=').nth(1))
                    .unwrap_or_default();
                println!("Access Token: {:?}", token);

                //Respond to close connection
                let response = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                break; //shut down server
            }
        }
    }
    return Ok(());
}

pub fn dropbox_access() {
    let client_id = "im68gew9aehy2pn".to_string();

    let client = BasicClient::new(
        ClientId::new(client_id),
        None,
        AuthUrl::new("https://www.dropbox.com/oauth2/authorize".to_string())
            .expect("Invalid authorization endpoint URL"),
        Some(TokenUrl::new("https://api.dropboxapi.com/oauth2/token".to_string())
            .expect("Invalid token endpoint URL")),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:3000".to_string()).unwrap(),
    );

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .url();
}
