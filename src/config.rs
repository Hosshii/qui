use std::{
    fs::{DirBuilder, File},
    io::prelude::*,
    path::PathBuf,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    server_url: String,
    client_id: String,
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

    pub fn set_client_id(&mut self, client_id: impl Into<String>) {
        self.client_id = client_id.into();
    }

    pub fn get_server_url(&self) -> &str {
        self.server_url.as_str()
    }
}

impl Default for Data {
    fn default() -> Self {
        Self {
            server_url: "https://traq-s-dev.tokyotech.org/api/v3".to_owned(),
            client_id: "xIwrarN2fZn4ikXBscU8YdA8ZcGGOQD2CczY".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub data: Data,

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
    use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode};
    use tui::{
        backend::{Backend, TermionBackend},
        layout::{Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Span, Spans, Text},
        widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
        Terminal,
    };

    use crate::utils::{Event, Events};

    use super::*;
    use std::io;

    pub struct StatefulList<T> {
        pub state: ListState,
        pub items: Vec<T>,
    }

    impl<T> StatefulList<T> {
        pub fn with_items(items: Vec<T>) -> StatefulList<T> {
            StatefulList {
                state: ListState::default(),
                items,
            }
        }

        pub fn next(&mut self) {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= self.items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }

        pub fn previous(&mut self) {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }

        pub fn unselect(&mut self) {
            self.state.select(None);
        }
    }

    impl<T> Default for StatefulList<T> {
        fn default() -> StatefulList<T> {
            StatefulList {
                state: ListState::default(),
                items: Vec::new(),
            }
        }
    }

    enum DisplayState {
        Quiet,
        SelectServer,
        InputUrl,
    }

    struct App<B>
    where
        B: Backend,
    {
        terminal: Terminal<B>,
        events: Events,
        display_state: DisplayState,
        config: Config,
    }

    impl<B> App<B>
    where
        B: Backend,
    {
        pub fn new(terminal: Terminal<B>, events: Events, dir_path: PathBuf) -> Self {
            Self {
                terminal,
                events,
                display_state: DisplayState::SelectServer,
                config: Config::new(Data::default(), dir_path),
            }
        }

        pub fn select_own_server(&mut self) {
            self.display_state = DisplayState::InputUrl;
        }

        pub fn set_server_url(&mut self, url: impl Into<String>) {
            self.config.data.set_server_url(url);
        }

        pub fn set_client_id(&mut self, client_id: impl Into<String>) {
            self.config.data.set_client_id(client_id);
        }

        pub fn set_quiet(&mut self) {
            self.display_state = DisplayState::Quiet;
        }
    }

    pub fn ui(dir_path: PathBuf) -> Result<Config> {
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        // let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let events = Events::default();
        let mut app = App::new(terminal, events, dir_path);
        app.terminal.clear()?;

        let data = vec![
            (
                "use default url (https://q.trap.jp/api/v3)",
                "https://q.trap.jp/api/v3",
                "6uT93VLLNjAfEkgX5IOYP4gHdW6p00dfgfPy",
            ),
            (
                "use dev server (https://traq-s-dev.tokyotech.org/api/v3)",
                "https://traq-s-dev.tokyotech.org/api/v3",
                "xIwrarN2fZn4ikXBscU8YdA8ZcGGOQD2CczY",
            ),
            ("set manually", "", ""),
        ];
        let mut stateful_list = StatefulList::with_items(data);

        let mut input = String::new();
        let mut input_mode = &mut InputMode::Normal;

        let mut client_id = String::new();
        let mut config_kind = &mut ConfigKind::ServerUrl;
        loop {
            match app.display_state {
                DisplayState::Quiet => break,
                DisplayState::SelectServer => draw_url_list(&mut app, &mut stateful_list)?,
                DisplayState::InputUrl => input_url(
                    &mut app,
                    &mut input_mode,
                    &mut input,
                    &mut config_kind,
                    &mut client_id,
                )?,
            }
        }
        app.terminal.clear()?;

        Ok(app.config)
    }

    fn draw_url_list<T>(
        app: &mut App<T>,
        stateful_list: &mut StatefulList<(&str, &str, &str)>,
    ) -> Result<()>
    where
        T: Backend,
    {
        app.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(1)
                .split(f.size());

            let items: Vec<ListItem> = stateful_list
                .items
                .iter()
                .enumerate()
                .map(|(idx, i)| {
                    let sp = Span::from(format!(" {} {}", idx + 1, i.0));
                    ListItem::new(sp).style(Style::default())
                })
                .collect();
            let list = List::new(items)
                .block(Block::default().title("choose "))
                .highlight_style(Style::default().bg(Color::LightBlue));
            f.render_stateful_widget(list, chunks[0], &mut stateful_list.state)
        })?;

        match app.events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') => app.set_quiet(),

                Key::Char(x) if x == '1' || x == '2' => {
                    app.set_server_url(stateful_list.items[(x as u8 - '1' as u8) as usize].1);
                    app.set_client_id(stateful_list.items[(x as u8 - '1' as u8) as usize].2);
                    app.set_quiet();
                }
                Key::Char('3') => {
                    app.select_own_server();
                }
                Key::Down | Key::Char('j') => {
                    stateful_list.next();
                }
                Key::Up | Key::Char('k') => {
                    stateful_list.previous();
                }
                Key::Char('\n') => {
                    if let Some(i) = stateful_list.state.selected() {
                        match i {
                            x @ 0..=1 => {
                                app.set_server_url(stateful_list.items[x].1);
                                app.set_client_id(stateful_list.items[x].2);
                                app.set_quiet();
                            }
                            2 => {
                                app.select_own_server();
                            }
                            x => {
                                unreachable!("{}", x)
                            }
                        }
                    }
                }

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
        Ok(())
    }

    #[derive(Debug)]
    enum InputMode {
        Normal,
        Editing,
    }

    enum ConfigKind {
        ServerUrl,
        ClientID,
    }

    fn input_url<T>(
        app: &mut App<T>,
        input_mode: &mut InputMode,
        server_url: &mut String,
        config_kind: &mut ConfigKind,
        client_id: &mut String,
    ) -> Result<()>
    where
        T: Backend,
    {
        use unicode_width::UnicodeWidthStr;
        app.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(5),
                        Constraint::Length(5),
                        Constraint::Max(0),
                    ]
                    .as_ref(),
                )
                .margin(1)
                .split(f.size());

            let (msg, style) = match input_mode {
                InputMode::Normal => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to exit, "),
                        Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to start editing, "),
                        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to set url."),
                    ],
                    Style::default().add_modifier(Modifier::RAPID_BLINK),
                ),
                InputMode::Editing => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to stop editing, "),
                    ],
                    Style::default(),
                ),
            };
            let mut text = Text::from(Spans::from(msg));
            text.patch_style(style);
            let help_message = Paragraph::new(text);
            f.render_widget(help_message, chunks[0]);

            let styles = match config_kind {
                ConfigKind::ServerUrl => (
                    Style::default().fg(Color::LightBlue),
                    Style::default().fg(Color::DarkGray),
                ),
                ConfigKind::ClientID => (
                    Style::default().fg(Color::DarkGray),
                    Style::default().fg(Color::LightBlue),
                ),
            };

            let server_url_widget = Paragraph::new(server_url.as_ref()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(styles.0)
                    .title("input server url"),
            );

            let client_id_widget = Paragraph::new(client_id.as_ref()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(styles.1)
                    .title("input client id"),
            );

            let idx = match config_kind {
                ConfigKind::ServerUrl => (1, server_url.as_str()),
                ConfigKind::ClientID => (2, client_id.as_str()),
            };
            match input_mode {
                InputMode::Normal => {}
                InputMode::Editing => {
                    f.set_cursor(
                        chunks[idx.0].x + idx.1.width() as u16 + 1,
                        // Move one line down, from the border to the input line
                        chunks[idx.0].y + 1,
                    );
                }
            }

            f.render_widget(server_url_widget, chunks[1]);
            f.render_widget(client_id_widget, chunks[2]);
        })?;

        match input_mode {
            InputMode::Normal => match app.events.next()? {
                Event::Input(k) => match k {
                    Key::Down | Key::Char('j') => {
                        *config_kind = ConfigKind::ClientID;
                    }
                    Key::Up | Key::Char('k') => {
                        *config_kind = ConfigKind::ServerUrl;
                    }
                    Key::Char('i') => {
                        *input_mode = InputMode::Editing;
                        app.events.disable_exit_key();
                    }
                    Key::Char('q') => app.set_quiet(),
                    Key::Char('\n') => {
                        app.set_server_url(server_url.clone());
                        app.set_client_id(client_id.clone());
                        app.set_quiet();
                    }
                    _ => {}
                },
                Event::Tick => {}
                _ => {}
            },
            InputMode::Editing => match app.events.next()? {
                Event::Input(k) => match k {
                    Key::Esc => {
                        *input_mode = InputMode::Normal;
                        app.events.enable_exit_key();
                    }

                    Key::Char(c) => {
                        let txt = match config_kind {
                            ConfigKind::ServerUrl => server_url,
                            ConfigKind::ClientID => client_id,
                        };
                        txt.push(c);
                    }
                    Key::Backspace => {
                        let txt = match config_kind {
                            ConfigKind::ServerUrl => server_url,
                            ConfigKind::ClientID => client_id,
                        };
                        txt.pop();
                    }
                    _ => {}
                },
                Event::Tick => {}
                Event::Message(_) => {}
            },
        }
        Ok(())
    }
}
