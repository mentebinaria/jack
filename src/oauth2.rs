// https://github.com/ramosbugs/oauth2-rs/blob/main/examples/google.rs

use oauth2::{basic::BasicClient, TokenResponse};
// Alternatively, this can be oauth2::curl::http_client or a custom.
use oauth2::reqwest::http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
};
use std::{io::{BufRead, BufReader, Write}, fs, path::Path};
use std::net::TcpListener;
use oauth2::url::Url;

#[inline(always)]
fn cache_token(token: &str) {
    fs::File::create(".oauth_tokens").unwrap().write_all(token.as_bytes()).unwrap();
}

// TODO: Add a `authenticate` table entry on the .toml file
pub fn authenticate<P: AsRef<Path>>(client_secrets: &P) -> String {
    if let Ok(token) = fs::read_to_string(".oauth_tokens") {
        return token;
    }
    
    // TODO: Add a proper way of parsing the file
    let parsed: serde_json::Value = serde_json::from_str(&fs::read_to_string(client_secrets).unwrap()).unwrap();
    let client_id = parsed["installed"]["client_id"].as_str().unwrap().to_string();
    let client_secret = parsed["installed"]["client_secret"].as_str().unwrap().to_string();
    let auth_uri = parsed["installed"]["auth_uri"].as_str().unwrap().to_string();
    let token_uri = parsed["installed"]["token_uri"].as_str().unwrap().to_string();

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_uri).unwrap(),
        Some(TokenUrl::new(token_uri).unwrap()),
    )
    // This example will be running its own server at localhost:8080.
    // See below for the server implementation.
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8080".to_string()).expect("Invalid redirect URL"),
    );

    // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/yt-analytics.readonly".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    println!(
        "Open this URL in your browser:\n{}\n",
        authorize_url
    );
    
    // A very naive implementation of the redirect server.
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    // Disable anoying clippy warning
    #[allow(clippy::manual_flatten)]
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

                let code_pair = url
                    .query_pairs()
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
                .request(http_client).unwrap();

            println!(
                "Google returned the following token:\n{:?}\n",
                token_response
            );

            let token = token_response.access_token().secret().to_string();
            cache_token(&token);
            return token;

            // The server will terminate itself after revoking the token.
        }
    }
    String::new()
}