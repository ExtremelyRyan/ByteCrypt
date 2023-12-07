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


pub fn google_access() -> anyhow::Result<()> {
    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        ClientId::new(
            env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
        ),
        None,
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .expect("Invalid authorization endpoint URL"),
        None,
        // Some(TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        //     .expect("Invalid token endpoint URL")),
    )
    .set_redirect_uri( //Use a local server to redirect
        RedirectUrl::new("http://localhost:3000".to_string()).expect("Invalid redirect URL"),
    );
    // .set_revocation_uri( //Auth 2.0 revocation
    //     RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
    //         .expect("Invalid revocation endpoint URL"),
    // );

    //Authorization URL to redirect the user
    let (authorize_url, csrf_token) = client
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

    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let code;
            let state;
            {
                let mut reader = BufReader::new(&stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();
                println!("{:?}", request_line);
                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();
                url.query_pairs().for_each(|p| println!("{:?}", p));

                let code_pair = url.query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .unwrap();

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());
                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .unwrap();

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());
            }

            let message = "Go back to your terminal :)";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            stream.write_all(response.as_bytes()).unwrap();

            println!("Google returned the following code:\n{}\n", code.secret());
            println!(
                "Google returned the following state:\n{} (expected `{}`)\n",
                state.secret(),
                csrf_token.secret()
            );

        //     // Exchange the code with a token.
        //     let token_response = client
        //         .exchange_code(code)
        //         .set_pkce_verifier(pkce_code_verifier)
        //         .request(http_client);

        //     println!(
        //         "Google returned the following token:\n{:?}\n",
        //         token_response
        //     );

        //     // Revoke the obtained token
        //     let token_response = token_response.unwrap();
        //     let token_to_revoke: StandardRevocableToken = match token_response.refresh_token() {
        //         Some(token) => token.into(),
        //         None => token_response.access_token().into(),
        //     };

        //     client
        //         .revoke_token(token_to_revoke)
        //         .unwrap()
        //         .request(http_client)
        //         .expect("Failed to revoke token");

        //     // The server will terminate itself after revoking the token.
        //     break;
        }
    }
    /*
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
            if request.starts_with("POST /token") {

                let mut reader = BufReader::new(&stream);
                println!("checkpoint");
                let mut headers = String::new();
                let mut content_length = 0;

                // Read headers
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" { break; }

                    if line.starts_with("Content-Length:") {
                        content_length = line.split_whitespace().nth(1).unwrap().parse::<usize>().unwrap();
                    }
                    headers.push_str(&line);
                }

                // Read the body based on the Content-Length
                let mut body = String::new();
                let mut body_buffer = vec![0_u8, content_length as u8];
                println!("Received body: {}", body);

                reader.read_exact(&mut body_buffer).unwrap();
                body.push_str(&String::from_utf8_lossy(&body_buffer));
                println!("checkpoint2");
                println!("{}", body);

                let token = body.split("&")
                    .find(|param| param.starts_with("access_token"))
                    .and_then(|param| param.split('=').nth(1))
                    .unwrap_or_default();
                println!("Access Token: {:?}", token);

                // Send a response back to the client
                let response = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
                break; //shut down server
            }
        }
    }*/
    return Ok(());
}
