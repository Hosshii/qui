use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
    Frame,
};

use crate::app::{App, AppState};

struct WidgetState {
    scroll: u8,
}

impl WidgetState {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }
}

struct MyWidget {
    chunk: Rect,
    widget: Box<dyn Widget>,
    state: WidgetState,
}

impl MyWidget {
    pub fn new(chunk: Rect, widget: Box<dyn Widget>) -> Self {
        Self {
            chunk,
            widget,
            state: WidgetState::new(),
        }
    }
}

fn get_chunk(f: Rect) -> Vec<Rect> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        // .margin(5)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(80)].as_ref())
        .split(f);
    chunks
}

pub fn draw_all<B: Backend>(f: &mut Frame<B>, app_state: &AppState, msg: &str) {
    let chunks = get_chunk(app_state.frame);
    let lhs = lhs();
    let rhs = rhs(app_state, msg.to_owned());

    f.render_widget(lhs, chunks[0]);
    f.render_widget(rhs, chunks[1])
}

pub fn draw_rhs<B: Backend>(f: &mut Frame<B>, app_state: &AppState, msg: &str) {
    let chunks = get_chunk(app_state.frame);
    let rhs = rhs(app_state, msg.to_owned());
    f.render_widget(rhs, chunks[1])
}

pub fn rhs(app_state: &AppState, msg: String) -> impl Widget {
    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    let paragraph = Paragraph::new(msg)
        .style(Style::default().bg(Color::White).fg(Color::Black))
        .block(create_block("Center, wrap"))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .scroll((0, 0));

    paragraph
}

fn lhs() -> impl Widget {
    let block = Block::default()
        .style(Style::default().bg(Color::White).fg(Color::Black))
        .title(Span::styled(
            "lhs",
            Style::default().add_modifier(Modifier::BOLD),
        ));
    block
}
