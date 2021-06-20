use std::{
    fs::{DirBuilder, File},
    io::prelude::*,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    server_url: String,
}

impl Data {
    pub fn save(&self, path: PathBuf) -> Result<()> {
        let serialized = serde_json::to_string(self).with_context(|| "serialize error")?;
        save_file(path, serialized.as_bytes())?;

        Ok(())
    }

    pub fn filename() -> &'static str {
        "config"
    }

    pub fn set_server_url(&mut self, url: impl Into<String>) {
        self.server_url = url.into();
    }
}

impl Default for Data {
    fn default() -> Self {
        Self {
            server_url: "https://traq-s-dev.tokyotech.org/api/v3".to_owned(),
        }
    }
}

pub struct Config {
    data: Data,

    dir_path: PathBuf,
}

impl Config {
    pub fn new(data: Data, dir_path: PathBuf) -> Self {
        Self { data, dir_path }
    }
    pub fn save(&self) -> Result<()> {
        if !self.dir_path.exists() {
            let mut builder = DirBuilder::new();

            builder.recursive(true).create(self.dir_path.as_path())?;
        }

        let mut config_path = self.dir_path.clone();
        config_path.push(Data::filename());
        self.data.save(config_path)?;

        Ok(())
    }

    pub fn load(dir_path: PathBuf) -> Result<Self> {
        let mut file_path = dir_path.clone();
        file_path.push(Data::filename());
        let mut file = File::open(file_path).with_context(|| "cannot open file")?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .with_context(|| "cannot read file")?;

        let deserialized: Data =
            serde_json::from_str(&buf).with_context(|| "cannot deserialize file")?;

        let res = Self::new(deserialized, dir_path);
        Ok(res)
    }
}

fn save_file(path: PathBuf, content: &[u8]) -> Result<()> {
    let mut file = File::create(path.as_path()).with_context(|| "cannot create file")?;
    file.write_all(content)
        .with_context(|| "write file error")?;
    Ok(())
}

pub mod ui {
    use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
    use tui::{
        backend::TermionBackend,
        layout::{Constraint, Direction, Layout},
        style::Style,
        text::Span,
        widgets::{Block, Borders, List, ListItem},
        Terminal,
    };

    use crate::utils::{Event, Events};

    use super::*;
    use std::io;

    pub fn ui() -> Result<()> {
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Setup event handlers
        let mut events = Events::default();

        let data = vec!["use default url (https://q.trap.jp)", "set url manually"];

        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .margin(1)
                    .split(f.size());

                let items: Vec<ListItem> = data
                    .iter()
                    .enumerate()
                    .map(|(idx, i)| {
                        let sp = Span::from(format!(" {} {}", idx + 1, i));
                        ListItem::new(sp).style(Style::default())
                    })
                    .collect();
                let list = List::new(items).block(Block::default().title("choose "));
                f.render_widget(list, chunks[0])
            })?;

            match events.next()? {
                Event::Input(key) => match key {
                    Key::Char('q') => break,

                    Key::Char('1') => {}

                    x => {
                        dbg!("{:?}", x);
                    }
                },
                Event::Tick => {
                    //     dbg!("tick");
                }
                Event::Message(msg) => {
                    dbg!("{}", msg);
                }
            }
        }

        Ok(())
    }
}
