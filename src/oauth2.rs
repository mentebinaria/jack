// https://github.com/ramosbugs/oauth2-rs/blob/main/examples/google.rs

use oauth2::{basic::BasicClient, TokenResponse, ClientSecret};
// Alternatively, this can be oauth2::curl::http_client or a custom.
use oauth2::reqwest::http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use oauth2::url::Url;

#[inline(always)]
fn cache_token(token: &str) {
    std::fs::File::create(".oauth_tokens").unwrap().write_all(token.as_bytes()).unwrap();
}

pub fn authenticate(oauth: &toml::value::Table) -> String {
    if let Ok(token) = std::fs::read_to_string(".oauth_tokens") {
        return token;
    }
    
    let client_secret = 
        oauth.get("client_secret").map(|client_id| ClientSecret::new(client_id.as_str().unwrap().to_owned()));
    let client_id = oauth["client_id"].as_str().unwrap().to_string();
    let auth_uri = oauth["auth_uri"].as_str().unwrap().to_string();
    let token_uri = oauth["token_uri"].as_str().unwrap().to_string();

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        ClientId::new(client_id),
        client_secret,
        // Some(ClientSecret::new(client_secret)),
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
    let (authorize_url, _) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/yt-analytics.readonly".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    println!(
        "If it didn't opened, open manually :\n{}\n",
        authorize_url
    );
    open::that(authorize_url.to_string()).map_err(|err| {
        println!(
            "ERROR: {err}\nCould not open url in your browser, try open manually",
        );
    }).unwrap();
    
    // A very naive implementation of the redirect server.
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    // Disable anoying clippy warning
    #[allow(clippy::manual_flatten)]
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let code;
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

                url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .unwrap();
            }

            let message = "Go back to your terminal :)";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            stream.write_all(response.as_bytes()).unwrap();

            // Exchange the code with a token.
            let token_response = client
                .exchange_code(code)
                .set_pkce_verifier(pkce_code_verifier)
                .request(http_client).unwrap();

            let token = token_response.access_token().secret().to_string();
            cache_token(&token);
            return token;

            // The server will terminate itself after revoking the token.
        }
    }
    panic!("Something failed")
}