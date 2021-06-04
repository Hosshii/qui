use termion::event::Key;
use tui::{backend::Backend, layout::Rect, Terminal};

use crate::{
    ui,
    utils::{Event, Events},
};

const MAX_CHANNEL_WINDOW_SIZE: usize = 5;
#[derive(Debug, PartialEq, Eq)]
enum ActiveBlock {
    Help,
    Channel([u8; MAX_CHANNEL_WINDOW_SIZE]),
    ChannelTree,
    Empty,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AppState {
    active_block: ActiveBlock,
    key: Key,
    should_quit: bool,
    pub frame: Rect,
}

impl AppState {
    pub fn new(frame: Rect) -> Self {
        Self {
            active_block: ActiveBlock::Empty,
            key: Key::Null,
            should_quit: false,
            frame,
        }
    }

    fn on_key(&mut self, k: Key) {
        self.key = k;
        match k {
            Key::Char(c) => match c {
                'q' if self.active_block == ActiveBlock::Empty => {
                    self.should_quit = true;
                }
                _ => {}
            },
            Key::Ctrl('c') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn key(&self) -> Key {
        self.key
    }
}

pub struct App<B>
where
    B: Backend,
{
    events: Events,
    terminal: Terminal<B>,
    state: AppState,
}

impl<B> App<B>
where
    B: Backend,
{
    pub fn new(events: Events, terminal: Terminal<B>) -> Self {
        let s = terminal.size().expect("get size err");
        Self {
            events,
            terminal,
            state: AppState::new(s),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = &mut self.state;
        self.terminal.draw(|f| ui::draw_all(f, &mut state, ""))?;
        let mut before_size = self.terminal.size()?;

        let mut msg = "".to_owned();
        loop {
            if before_size != self.terminal.size()? {
                self.on_size_change(&msg)?;
            }

            // これを下に動かすとサイズが変わらなくなる
            before_size = self.terminal.size()?;

            match self.events.next()? {
                Event::Input(key) => {
                    self.on_key(key);
                    // self.terminal.draw(|f| ui::draw(f, &mut state))?;
                }
                Event::Tick => {
                    // todo!()
                    // self.terminal.draw(|f| ui::draw(f, &mut state))?;
                }
                Event::Message(s) => {
                    let mut state = &mut self.state;
                    self.terminal.draw(|f| ui::draw_all(f, &mut state, &s))?;
                    msg = s;
                }
            }

            if self.should_quit() {
                break;
            }
        }
        Ok(())
    }

    fn on_key(&mut self, key: Key) {
        self.state.on_key(key);
    }

    fn should_quit(&self) -> bool {
        self.state.should_quit
    }

    fn on_size_change(&mut self, msg: &str) -> std::io::Result<()> {
        self.state.frame = self.terminal.size()?;
        let mut state = &mut self.state;
        self.terminal.draw(|f| ui::draw_all(f, &mut state, &msg))?;
        Ok(())
    }
}

pub struct WidgetState {
    selected: u8,
    msg_state: Vec<MessageState>,
}

impl WidgetState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            msg_state: Vec::new(),
        }
    }
}

struct MessageState {
    scroll: u64,
    channel_id: String,
}

impl MessageState {
    pub fn new(scroll: u64, channel_id: String) -> Self {
        Self { scroll, channel_id }
    }
}
