use anyhow::{Context, Result};
use qui::{
    cli::{clap_app, handle},
    token::{self, TraqOAuthParam},
    // utils::Events,
};
use rust_traq::apis::configuration::Configuration;
use std::{env, io, path::PathBuf};
// use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
// use tui::{backend::TermionBackend, Terminal};

#[tokio::main]
async fn main() -> Result<()> {
    // let stdout = io::stdout().into_raw_mode()?;
    // let stdout = MouseTerminal::from(stdout);
    // let stdout = AlternateScreen::from(stdout);
    // let backend = TermionBackend::new(stdout);
    // let terminal = Terminal::new(backend)?;

    // let events = Events::default();
    // let mut app = App::new(events, terminal);

    // if let Err(e) = app.run() {
    //     eprintln!("{}", e);
    // }

    let port = 8080;
    let client_id = &env::var("TRAQ_CLIENT_ID").expect("TRAQ_CLIENT_ID must not be blunk");

    let mut conf = &mut Configuration::default();
    conf.base_path = "https://traq-s-dev.tokyotech.org/api/v3".to_owned();

    let mut traq_oauth = TraqOAuthParam::new(&conf, &&client_id, None);

    let token_path = get_token_path()?;
    let token_path = token_path.as_path();

    match token::get_cached_token(&token_path) {
        Some(token) => {
            conf.oauth_access_token = Some(token);
        }
        None => match token::redirect_uri_web_server(&mut traq_oauth, port) {
            Ok(url) => {
                let tk = token::get_token(&mut traq_oauth, url).await?;
                conf.oauth_access_token = Some(tk.access_token);
                token::verify_token(conf)
                    .await
                    .with_context(|| "verification error")?;
                token::store_token(&token_path, conf.oauth_access_token.as_ref().unwrap())?;
            }
            Err(_) => {
                println!("Starting webserver failed. Continuing with manual authentication");
                token::request_token(&mut traq_oauth);
                println!("Enter the URL you were redirected to: ");
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        let tk = token::get_token(&mut traq_oauth, input).await?;
                        conf.oauth_access_token = Some(tk.access_token);
                        token::verify_token(conf)
                            .await
                            .with_context(|| "verification error")?;
                        token::store_token(&token_path, conf.oauth_access_token.as_ref().unwrap())?;
                    }
                    Err(_) => todo!(),
                }
            }
        },
    }

    let app = clap_app::clap_app();
    let matches = app.get_matches();
    if let Some(cmd) = matches.subcommand_name() {
        let m = matches.subcommand_matches(cmd).unwrap();
        handle::handle_matches(conf, m, cmd).await?;
    }

    Ok(())
}

fn get_token_path() -> Result<PathBuf> {
    let home = env::var("HOME")?;

    let mut path = PathBuf::new();
    path.push(home);
    path.push(".config");
    path.push("qui");
    path.push("token.txt");
    Ok(path)
}
