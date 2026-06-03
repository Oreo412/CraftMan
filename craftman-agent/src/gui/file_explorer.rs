use std::io::Stdout;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, FrameExt as _, Paragraph},
};
use ratatui_explorer::{FileExplorer, Theme};

pub fn blocking_file_selection(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(String, String)> {
    let theme = Theme::default()
        .add_default_title()
        .with_block(Block::default().borders(Borders::ALL))
        .with_highlight_item_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .with_highlight_dir_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .with_highlight_symbol(">> ");

    let mut explorer = FileExplorer::new()?;
    explorer.set_theme(theme);

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(frame.area());
            let widget = explorer.widget();
            frame.render_widget_ref(widget, chunks[0]);
            let keys = Paragraph::new(
                "Esc: Go back | Arrow Keys: Navigate files/directories | Enter: Select Server File",
            )
            .block(Block::default().borders(Borders::ALL));

            frame.render_widget(keys, chunks[1]);
        })?;

        let event = event::read()?;

        match &event {
            Event::Key(key) if key.code == KeyCode::Enter => {
                tracing::info!("File selected. Exiting file_selection");
                let current = explorer.current();

                return Ok((current.name.clone(), explorer.cwd().display().to_string()));
            }

            Event::Key(key) if key.code == KeyCode::Esc => {
                anyhow::bail!("file selection cancelled");
            }

            _ => {
                explorer.handle(&event)?;
            }
        }
    }
}
