use oauth2::{
    basic::BasicClient, 
    revocation::StandardRevocableToken, 
    reqwest::http_client,
    TokenResponse,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    RevocationUrl, Scope, TokenUrl,
};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;



pub fn google_drive_access() {
    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        ClientId::new(
            env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
        ),
        None,
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .expect("Invalid authorization endpoint URL"),
        Some(TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
            .expect("Invalid token endpoint URL")),
    )
    .set_redirect_uri( //Use a local server to redirect
        RedirectUrl::new("http://localhost:3000".to_string()).expect("Invalid redirect URL"),
    )
    .set_revocation_uri( //Auth 2.0 revocation
        RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
            .expect("Invalid revocation endpoint URL"),
    );

    //PKCE - https://oauth.net/2/pkce/
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    //Authorization URL to redirect the user
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/drive.file".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    println!(
        "Open this URL to authorize this application:\n{}\n",
        authorize_url.to_string()
    );

    // A very naive implementation of the redirect server.
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let code;
            let state;
            {
                let mut reader = BufReader::new(&stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

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
                csrf_state.secret()
            );

            // Exchange the code with a token.
            let token_response = client
                .exchange_code(code)
                .set_pkce_verifier(pkce_code_verifier)
                .request(http_client);

            println!(
                "Google returned the following token:\n{:?}\n",
                token_response
            );

            // Revoke the obtained token
            let token_response = token_response.unwrap();
            let token_to_revoke: StandardRevocableToken = match token_response.refresh_token() {
                Some(token) => token.into(),
                None => token_response.access_token().into(),
            };

            client
                .revoke_token(token_to_revoke)
                .unwrap()
                .request(http_client)
                .expect("Failed to revoke token");

            // The server will terminate itself after revoking the token.
            break;
        }
    }
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
