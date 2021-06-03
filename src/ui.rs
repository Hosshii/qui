use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppState};

fn get_chunk<B: Backend>(f: &mut Frame<B>) -> Vec<Rect> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());
    chunks
}

pub fn draw_all<B: Backend>(f: &mut Frame<B>, app_state: &AppState, msg: &str) {
    let chunks = get_chunk(f);
    let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Black));
    f.render_widget(block, f.size());

    draw_text(f, app_state, msg);
}

pub fn draw_text<B: Backend>(f: &mut Frame<B>, app_state: &AppState, msg: &str) {
    let chunks = get_chunk(f);

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

    f.render_widget(paragraph, chunks[1]);
}
