use crate::json_parse;

use oauth2::{basic::BasicClient, reqwest::http_client, TokenResponse, StandardRevocableToken, RevocableToken};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    RevocationUrl, Scope, TokenUrl,
};
use url::Url;
use std::env;
use std::io::{BufRead, BufReader, Write, Error};
use std::net::TcpListener;
use std::fs;
use std::process::exit;

struct Secret {
    client_id: String,
    project_id: String,
    auth_uri: String,
    token_uri: String,
    auth_provider_x509_cert_url: String,
    client_secret: String,
}

pub fn get_secret() -> Result<(String, String), Error> {
    let content = fs::read_to_string("client_secret.json")?;

    let p = match json_parse::deserialize(&content) {
        Ok(p) => p,
        Err(e) => {
            panic!("{:?}", e);
        },
    };

    Ok((p["web"]["client_id"].as_str().unwrap().to_string(), p["web"]["client_secret"].as_str().unwrap().to_string()))
}

pub fn get_oauth2_token() -> Result<String, Error> {
    let secret = get_secret()?;

    let google_client_id = ClientId::new(secret.0);
    let google_client_secret = ClientSecret::new(secret.1);
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("Invalid token endpoint URL");

    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:8080".to_string()).expect("Invalid redirect URL"),
        )
        .set_revocation_uri(
            RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
                .expect("Invalid revocation endpoint URL"),
        );

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://mail.google.com/".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    println!(
        "Open this URL in your browser:\n{}\n",
        authorize_url.to_string()
    );

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let mut token = String::new();

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

            // println!("Google returned the following code:\n{}\n", code.secret());
            // println!(
            //     "Google returned the following state:\n{} (expected `{}`)\n",
            //     state.secret(),
            //     csrf_state.secret()
            // );

            let token_response = client
                .exchange_code(code)
                .set_pkce_verifier(pkce_code_verifier)
                .request(http_client);

            // println!(
            //     "Google returned the following token:\n{:?}\n",
            //     token_response
            // );

            let token_response = token_response.unwrap();

            let token_to_revoke: StandardRevocableToken = match token_response.refresh_token() {
                Some(token) => {
                    // println!("token: {}", token.secret());
                    token.into()
                },
                None => {
                    // println!("token: {}", token_response.access_token().secret());
                    token_response.access_token().into()
                },
            };

            token = token_to_revoke.secret().to_string();

            // revoke
            // client
            //     .revoke_token(token_to_revoke)
            //     .unwrap()
            //     .request(http_client)
            //     .expect("Failed to revoke token");

            break;
        }
    }

    Ok(token)
}
