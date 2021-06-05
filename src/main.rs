use qui::{
    app::App,
    cli::{clap_app, handle},
    utils::Events,
};
use std::io;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let app = clap_app::clap_app();
    let matches = app.get_matches();
    if let Some(cmd) = matches.subcommand_name() {
        let m = matches.subcommand_matches(cmd).unwrap();
        handle::handle_matches(m, cmd);
    }

    Ok(())
}
