use std::io::{self, Read};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, SendError, Sender};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Message(String),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    ignore_exit_key: Arc<AtomicBool>,
    tick_handle: thread::JoinHandle<()>,
    req_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                        if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                            return;
                        }
                    }
                }
            })
        };
        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(config.tick_rate);
            })
        };

        let req_handle = {
            let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
            thread::spawn(move || {
                for stream in listener.incoming() {
                    let stream = stream.unwrap();

                    if handle_connection(stream, tx.clone()).is_err() {
                        break;
                    }
                }
            })
        };

        Events {
            rx,
            ignore_exit_key,
            input_handle,
            tick_handle,
            req_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed);
    }
}

fn handle_connection(
    mut stream: TcpStream,
    tx: Sender<Event<Key>>,
) -> Result<(), SendError<Event<Key>>> {
    let mut buffer = [0; 1024];

    let _ = stream.read(&mut buffer).unwrap();
    let s = String::from_utf8_lossy(&buffer[..]).to_string();
    tx.send(Event::Message(s))?;

    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    Ok(())
}
