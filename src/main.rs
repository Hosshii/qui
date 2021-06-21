use anyhow::{bail, Context, Result};
use clap::Shell;
use qui::{
    cli::{clap_app, handle},
    config::{self, Config},
    token::{self, TraqOAuthParam},
};
use rust_traq::apis::configuration::Configuration;
use std::{env, io, path::PathBuf};
// use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
// use tui::{backend::TermionBackend, Terminal};
const TRAQ_CLIENT_ID: &str = "xIwrarN2fZn4ikXBscU8YdA8ZcGGOQD2CczY";

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
    let conf = match Config::load(get_conf_path()?) {
        Ok(c) => c,
        Err(e) => {
            println!("cannot load config. start ui {}", e);
            config::ui::ui(get_conf_path()?).with_context(|| "cannot set config")?
        }
    };

    let port = 8080;
    let client_id = TRAQ_CLIENT_ID;

    let mut api_conf = &mut Configuration::default();
    api_conf.base_path = conf.data.get_server_url().to_owned();

    let mut traq_oauth = TraqOAuthParam::new(&api_conf, &client_id, None);

    let token_path = get_token_path()?;
    let token_path = token_path.as_path();

    match token::get_cached_token(&token_path) {
        Some(token) => {
            api_conf.oauth_access_token = Some(token);
        }
        None => match token::redirect_uri_web_server(&mut traq_oauth, port) {
            Ok(url) => {
                let tk = token::get_token(&mut traq_oauth, url).await?;
                api_conf.oauth_access_token = Some(tk.access_token);
                token::verify_token(api_conf)
                    .await
                    .with_context(|| "verification error")?;
                token::store_token(&token_path, api_conf.oauth_access_token.as_ref().unwrap())?;
            }
            Err(_) => {
                println!("Starting webserver failed. Continuing with manual authentication");
                token::request_token(&mut traq_oauth);
                println!("Enter the URL you were redirected to: ");
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        let tk = token::get_token(&mut traq_oauth, input).await?;
                        api_conf.oauth_access_token = Some(tk.access_token);
                        token::verify_token(api_conf)
                            .await
                            .with_context(|| "verification error")?;
                        token::store_token(
                            &token_path,
                            api_conf.oauth_access_token.as_ref().unwrap(),
                        )?;
                    }
                    Err(_) => todo!(),
                }
            }
        },
    }

    conf.save().with_context(|| "cannot save config")?;

    let mut app = clap_app::clap_app();
    let matches = app.clone().get_matches();

    // completions
    if let Some(s) = matches.value_of("completions") {
        let shell = match s {
            "fish" => Shell::Fish,
            "bash" => Shell::Bash,
            "zsh" => Shell::Zsh,
            "power-shell" => Shell::PowerShell,
            "elvish" => Shell::Elvish,
            _ => bail!("no completions avaible for '{}'", s),
        };
        app.gen_completions_to(env!("CARGO_BIN_NAME"), shell, &mut io::stdout());
        return Ok(());
    }

    if matches.is_present("set-config") {
        let config = config::ui::ui(get_conf_path()?)?;
        config.save()?;
        return Ok(());
    }

    if matches.is_present("show-config") {
        let config = Config::load(get_conf_path()?)?;
        println!("server url: {}", config.data.get_server_url());
        return Ok(());
    }

    if let Some(cmd) = matches.subcommand_name() {
        let m = matches.subcommand_matches(cmd).unwrap();
        handle::handle_matches(api_conf, m, cmd).await?;
    }

    Ok(())
}

fn get_token_path() -> Result<PathBuf> {
    let mut path = get_conf_path()?;
    path.push("token.txt");
    Ok(path)
}

fn get_conf_path() -> Result<PathBuf> {
    let home = env::var("HOME").with_context(|| "cannot get home dir")?;

    let mut path = PathBuf::new();
    path.push(home);
    path.push(".config");
    path.push("qui");
    Ok(path)
}
