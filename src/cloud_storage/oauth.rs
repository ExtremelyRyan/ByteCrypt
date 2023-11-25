// use oauth2::{
//     basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
//     TokenUrl,
// };

///Holds the user credentials for the session
struct UserCredentials {
    ///Grants access to the user account
    access_token: String,
    ///Type of token
    token_type: String,
    ///(Optional) -> get new access token when current one expires
    refresh_token: String,
    ///(Optional) -> Lifetime of token in seconds
    expires_in: Option<u64>,
    ///Scope of access and permissions granted
    scope: Option<String>,
}
