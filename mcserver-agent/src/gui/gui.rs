use std::{io, time::Duration};

use crate::{
    gui::{
        app::{App, AppState},
        file_explorer,
        gui_actions::ConfigRequest,
    },
    mods::configs::Configs,
};
use crossterm::{
    event::{self, Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::{FutureExt, StreamExt};
use protocol::agentactions::AgentActions;
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use anyhow::Result;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    time::interval,
};
use tui_file_explorer::FileExplorer;

pub enum GuiEvents {
    Validate(String),
    Validated,
    ServerStarted,
    ServerStopped,
    AddStdoutLine(String),
}

pub async fn handler(
    config: Configs,
    tui_to_agent: UnboundedSender<ConfigRequest>,
    mut tui_from_agent: UnboundedReceiver<GuiEvents>,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config, tui_to_agent);

    terminal.clear()?;
    terminal.draw(|f| ui(f, &app))?;

    let mut reader = EventStream::new();

    let mut tick = interval(Duration::from_millis(33)); // ~30 FPS

    loop {
        tokio::select! {
            Some(event) = tui_from_agent.recv() => {
                match event {
                    GuiEvents::AddStdoutLine(line) => app.stdout.push_front(line),
                    GuiEvents::Validate(key) => app.start_validation(key),
                    GuiEvents::Validated => app.complete_validation(),
                    GuiEvents::ServerStarted => app.server_running = true,
                    GuiEvents::ServerStopped => app.server_running = false,
                }

            }

            maybe_event = reader.next().fuse() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') => {break;}
                                KeyCode::Char('c') => {
                                    if let Ok((file, directory)) = file_explorer::file_selection(&mut terminal) &&
                                        let Err(e) = app.edit_config(app.config.clone().set_dir(directory).set_jar(file)).await {
                                            tracing::error!("Error editing selected file: {}", e);
                                    }
                                }
                                _ => {}
                            }

                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) => {
                        // handle error if needed
                    }
                    None => {
                        // stream ended (rare)
                        break;
                    }
                }
            }
            _ = tick.tick() => {
                terminal.draw(|f| ui(f, &app))?;
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(frame: &mut Frame, app: &App) {
    frame.render_widget(Clear, frame.area());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let status = Paragraph::new(if app.server_running {
        "Server is running"
    } else {
        "Server is off"
    })
    .block(
        Block::default()
            .title("Craftman Agent")
            .borders(Borders::ALL),
    );

    frame.render_widget(status, chunks[0]);

    let config_text: Vec<Line<'_>> = vec![
        Span::from(format!("Directory: {}", app.config.dir)),
        Span::from(format!("Min Memory: {}", app.config.xms)),
        Span::from(format!("Max Memory: {}", app.config.xmx)),
        Span::from(format!("Server Jar: {}", app.config.jar)),
    ]
    .iter()
    .map(|j| Line::from(j.clone()))
    .collect();

    let config = Paragraph::new(config_text).block(
        Block::default()
            .title("Configuration")
            .borders(Borders::ALL),
    );

    frame.render_widget(config, chunks[1]);

    let stdout_lines: Vec<Line<'_>> = app.stdout.iter().map(|l| Line::from(l.as_str())).collect();

    let stdout = Paragraph::new(stdout_lines)
        .block(
            Block::default()
                .title("Server Output")
                .borders(Borders::ALL),
        )
        .scroll((app.scroll, 0));

    frame.render_widget(stdout, chunks[2]);

    let keys = Paragraph::new("q: quit | s: start | x: stop | r: restart")
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(keys, chunks[3]);

    if let AppState::Validate(key) = &app.state {
        let area = frame.area();

        let popup_area = area.centered(Constraint::Percentage(60), Constraint::Percentage(30));

        let popup = Paragraph::new(format!("Enter key into Discord:\n {}", key))
            .block(Block::default().title("Popup").borders(Borders::ALL));

        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup, popup_area);
    }
}
