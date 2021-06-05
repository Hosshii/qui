use qui::{
    app::App,
    cli::{clap_app, handle},
    token::{self, TraqOAuthParam},
    utils::Events,
};
use rust_traq::apis::configuration::Configuration;
use std::io;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let mut conf = Configuration::default();
    let mut traq_oauth = TraqOAuthParam::new(&conf, env!("TRAQ_CLIENT_ID"), None);
    match token::redirect_uri_web_server(&mut traq_oauth, port) {
        Ok(mut url) => {
            let tk = token::get_token(&mut traq_oauth, url).await;
            dbg!("{:?}", &tk);
            conf.oauth_access_token = Some(tk.access_token);
        }
        Err(()) => {
            println!("Starting webserver failed. Continuing with manual authentication");
            token::request_token(&mut traq_oauth);
            println!("Enter the URL you were redirected to: ");
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let tk = token::get_token(&mut traq_oauth, input).await;
                    dbg!("{:?}", &tk);
                    conf.oauth_access_token = Some(tk.access_token);
                }
                Err(_) => (),
            }
        }
    }

    let app = clap_app::clap_app();
    let matches = app.get_matches();
    if let Some(cmd) = matches.subcommand_name() {
        let m = matches.subcommand_matches(cmd).unwrap();
        handle::handle_matches(m, cmd);
    }

    Ok(())
}
