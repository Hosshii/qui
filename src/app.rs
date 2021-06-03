use termion::event::Key;
use tui::{backend::Backend, Terminal};

use crate::{
    ui,
    utils::{Event, Events},
};

const MAX_CHANNEL_WINDOW_SIZE: usize = 5;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveBlock {
    Help,
    Channel([u8; MAX_CHANNEL_WINDOW_SIZE]),
    ChannelTree,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppState {
    active_block: ActiveBlock,
    key: Key,
    should_quit: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_block: ActiveBlock::Empty,
            key: Key::Null,
            should_quit: false,
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
        Self {
            events,
            terminal,
            state: AppState::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut state = self.state;
            self.terminal.draw(|f| ui::draw(f, &mut state))?;

            match self.events.next()? {
                Event::Input(key) => self.on_key(key),

                Event::Tick => {
                    // todo!()
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
