use qui::{app::App, utils::Events};
use std::io;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let events = Events::default();
    let mut app = App::new(events, terminal);

    if let Err(e) = app.run() {
        eprintln!("{}", e);
    }

    Ok(())
}
