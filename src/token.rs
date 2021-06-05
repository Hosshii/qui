use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

use rand::{self, distributions::Alphanumeric, Rng};
use rust_traq::{
    apis::{configuration, oauth2_api::post_o_auth2_token},
    models::{OAuth2Prompt, OAuth2ResponseType, OAuth2Token},
};

pub struct TraqOAuthParam<'a> {
    configuration: &'a configuration::Configuration,
    client_id: &'a str,
    response_type: Option<OAuth2ResponseType>,
    redirect_uri: Option<&'a str>,
    scope: Option<&'a str>,
    state: Option<&'a str>,
    code_challenge: Option<&'a str>,
    code_challenge_method: Option<&'a str>,
    nonce: Option<&'a str>,
    prompt: Option<OAuth2Prompt>,
}

impl<'a> TraqOAuthParam<'a> {
    pub fn new(
        configuration: &'a configuration::Configuration,
        client_id: &'a str,
        // redirect_uri: Option<&'a str>,
        state: Option<&'a str>,
    ) -> Self {
        Self {
            configuration,
            client_id,
            response_type: Some(OAuth2ResponseType::Code),
            redirect_uri: None,
            scope: None,
            state,
            code_challenge: None,
            code_challenge_method: None,
            nonce: None,
            prompt: None,
        }
    }

    pub fn get_authorize_url(&mut self, state: Option<&str>) -> String {
        let mut url = format!("{}/oauth2/authorize", &self.configuration.base_path);

        url += "?response_type=code";

        url += &format!("&client_id={}", self.client_id);
        if let Some(local_var_str) = self.redirect_uri {
            url += &format!("&redirect_uri={}", local_var_str);
        }
        if let Some(local_var_str) = self.scope {
            url += &format!("&scope={}", local_var_str);
        }
        if let Some(local_var_str) = state {
            url += &format!("&state={}", local_var_str);
        }
        if let Some(local_var_str) = self.code_challenge {
            url += &format!("&code_challenge={}", local_var_str);
        }
        if let Some(local_var_str) = self.code_challenge_method {
            url += &format!("&code_challenge_method={}", local_var_str);
        }
        if let Some(local_var_str) = self.nonce {
            url += &format!("&nonce={}", local_var_str);
        }
        if let Some(local_var_str) = self.prompt {
            url += &format!("&prompto={}", local_var_str);
        }
        url
    }
}

pub fn redirect_uri_web_server(traq_oauth: &mut TraqOAuthParam, port: u16) -> Result<String, ()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port));

    match listener {
        Ok(listener) => {
            request_token(traq_oauth);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        if let Some(url) = handle_connection(stream) {
                            return Ok(url);
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                };
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    Err(())
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
    let state = generate_random_string(16);
    let auth_url = traq_oauth.get_authorize_url(Some(&state));
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

pub async fn get_token(traq_oauth: &mut TraqOAuthParam<'_>, mut url: String) -> OAuth2Token {
    match parse_response_code(&mut url) {
        Some(code) => {
            match post_o_auth2_token(
                traq_oauth.configuration,
                "authorization_code",
                Some(&code),
                traq_oauth.redirect_uri,
                Some(traq_oauth.client_id),
                None,
                None,
                None,
                traq_oauth.scope,
                None,
                None,
            )
            .await
            {
                Ok(token) => return token,
                Err(e) => {
                    dbg!("{}", e);
                    todo!();
                }
            }
        }
        None => {
            todo!()
        }
    }
}

pub fn parse_response_code(url: &mut str) -> Option<String> {
    url.split("?code=")
        .nth(1)
        .and_then(|strs| strs.split('&').next())
        .map(|s| s.to_owned())
}
