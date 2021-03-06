#![allow(dead_code)]

use data_encoding::BASE64URL_NOPAD;
use ring::digest::{self, SHA256};
use std::{
    fs::{self, DirBuilder, File},
    io::prelude::*,
    net::{TcpListener, TcpStream},
    path::Path,
};

use anyhow::{bail, Context, Result};
use rand::{self, distributions::Alphanumeric, Rng};
use rust_traq::{
    apis::{
        configuration::{self, Configuration},
        me_api, oauth2_api,
    },
    models::{OAuth2Prompt, OAuth2ResponseType, OAuth2Token},
};

pub struct TraqOAuthParam<'a> {
    configuration: &'a configuration::Configuration,
    client_id: String,
    response_type: Option<OAuth2ResponseType>,
    redirect_uri: Option<String>,
    scope: Option<String>,
    state: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
    nonce: Option<String>,
    prompt: Option<OAuth2Prompt>,
}

impl<'a> TraqOAuthParam<'a> {
    pub fn new(
        configuration: &'a configuration::Configuration,
        client_id: String,
        code_verifier: String,
    ) -> Self {
        let state = Some(generate_random_string(16));
        let code_challenge = digest::digest(&SHA256, code_verifier.as_bytes());
        let code_challenge: String = BASE64URL_NOPAD.encode(code_challenge.as_ref());

        let code_challenge = Some(code_challenge);
        let code_challenge_method = Some("S256".to_owned());
        Self {
            configuration,
            client_id,
            response_type: Some(OAuth2ResponseType::Code),
            redirect_uri: None,
            scope: None,
            state,
            code_challenge,
            code_challenge_method,
            nonce: None,
            prompt: None,
        }
    }

    pub fn get_authorize_url(&mut self) -> String {
        let mut url = format!("{}/oauth2/authorize", &self.configuration.base_path);

        url += "?response_type=code";

        url += &format!("&client_id={}", self.client_id);
        if let Some(ref local_var_str) = self.redirect_uri {
            url += &format!("&redirect_uri={}", local_var_str);
        }
        if let Some(ref local_var_str) = self.scope {
            url += &format!("&scope={}", local_var_str);
        }
        if let Some(ref local_var_str) = self.state {
            url += &format!("&state={}", local_var_str);
        }
        if let Some(ref local_var_str) = self.code_challenge {
            url += &format!("&code_challenge={}", local_var_str);
        }
        if let Some(ref local_var_str) = self.code_challenge_method {
            url += &format!("&code_challenge_method={}", local_var_str);
        }
        if let Some(ref local_var_str) = self.nonce {
            url += &format!("&nonce={}", local_var_str);
        }
        if let Some(ref local_var_str) = self.prompt {
            url += &format!("&prompto={}", local_var_str);
        }
        url
    }
}

pub fn redirect_uri_web_server(traq_oauth: &mut TraqOAuthParam, port: u16) -> Result<String> {
    let addr = format!("127.0.0.1:{}", port);
    let listener =
        TcpListener::bind(&addr).with_context(|| format!("cannot bind address {}", addr))?;

    request_token(traq_oauth);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Some(url) = handle_connection(stream) {
                    return Ok(url);
                }
            }
            Err(e) => {
                bail!("{}", e)
            }
        };
    }

    bail!("cannot handle")
}

fn handle_connection(mut stream: TcpStream) -> Option<String> {
    // The request will be quite large (> 512) so just assign plenty just in case
    let mut buffer = [0; 1000];
    let _ = stream.read(&mut buffer).unwrap();

    // convert buffer into string and 'parse' the URL
    match String::from_utf8(buffer.to_vec()) {
        Ok(request) => {
            let split: Vec<&str> = request.split_whitespace().collect();

            if split.len() > 1 {
                respond_with_success(stream);
                return Some(split[1].to_string());
            }

            respond_with_error("Malformed request".to_string(), stream);
        }
        Err(e) => {
            respond_with_error(format!("Invalid UTF-8 sequence: {}", e), stream);
        }
    };

    None
}

fn respond_with_success(mut stream: TcpStream) {
    let contents = include_str!("redirect_uri.html");

    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn respond_with_error(error_message: String, mut stream: TcpStream) {
    println!("Error: {}", error_message);
    let response = format!(
        "HTTP/1.1 400 Bad Request\r\n\r\n400 - Bad Request - {}",
        error_message
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub fn request_token(traq_oauth: &mut TraqOAuthParam) {
    let auth_url = traq_oauth.get_authorize_url();
    match webbrowser::open(&auth_url) {
        Ok(_) => println!("Opened {} in your browser", auth_url),
        Err(why) => eprintln!("Error {:?};Please navigate here [{:?}] ", why, auth_url),
    }
}

pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub async fn get_token(
    traq_oauth: &mut TraqOAuthParam<'_>,
    mut url: String,
    code_verifier: Option<&str>,
) -> Result<OAuth2Token> {
    let code = parse_response_code(&mut url).with_context(|| "parse error")?;

    let token = oauth2_api::post_o_auth2_token(
        &traq_oauth.configuration,
        "authorization_code",
        Some(&code),
        traq_oauth.redirect_uri.as_deref(),
        Some(traq_oauth.client_id.as_ref()),
        code_verifier,
        None,
        None,
        traq_oauth.scope.as_deref(),
        None,
        None,
    )
    .await
    .with_context(|| "post token error")?;
    Ok(token)
}

pub fn parse_response_code(url: &mut str) -> Option<String> {
    url.split("?code=")
        .nth(1)
        .and_then(|strs| strs.split('&').next())
        .map(|s| s.to_owned())
}

const TOKEN_LENGTH: usize = 36;
pub fn get_cached_token(path: &Path) -> Option<String> {
    let display = path.display();
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(why) => {
            eprintln!("couldn't open {}: {:?}", display, why.to_string());
            return None;
        }
    };
    let mut token_info_string = String::new();
    match file.read_to_string(&mut token_info_string) {
        Err(why) => {
            eprintln!("couldn't read {}: {}", display, why.to_string());
            None
        }
        Ok(s) => {
            if s != TOKEN_LENGTH {
                eprintln!("token size is invalid. expected 34, found {}", s);
                None
            } else {
                Some(token_info_string)
            }
        }
    }
}

pub fn store_token(path: &Path, token: &str) -> Result<()> {
    if token.len() != TOKEN_LENGTH {
        eprintln!("token size is invalid. expected 34, found {}", token.len());
        bail!("invalid token length")
    }

    let display = path.display();

    let mut builder = DirBuilder::new();
    if let Some(parent) = path.parent() {
        builder.recursive(true).create(parent)?;
    }
    let mut file = File::create(&path).with_context(|| "cannot create file")?;

    file.write_all(token.as_bytes())
        .with_context(|| "write error")?;

    println!("token is stored in {}", display);
    Ok(())
}

pub fn delete_token(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("cannot remove token file: {}", path.display()))?;
    }
    Ok(())
}

pub async fn verify_token(conf: &Configuration) -> Result<()> {
    me_api::get_my_user_tags(conf).await?;
    Ok(())
}
