use crate::{gui::app::App, mods::configs::Configs};
use crossterm::event::{self, Event, EventStream, KeyEventKind};
use futures::{FutureExt, StreamExt};
use protocol::agentactions::AgentActions;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tui_file_explorer::FileExplorer;

pub fn build_handler(
    config: &Configs,
    directory: String,
    file: String,
) -> UnboundedSender<GuiEvents> {
    let (gui_sender, gui_receiver) = mpsc::unbounded_channel::<GuiEvents>();
    let (agent_sender, agent_receiver) = mpsc::unbounded_channel::<AgentActions>();
    tokio::spawn(handler(config, agent_sender, gui_receiver, directory, file));
    gui_sender
}

pub enum GuiEvents {
    Validate(String),
    Validated,
    ServerStarted,
    ServerStopped,
    AddStdoutLine(String),
}

async fn handler(
    config: &Configs,
    agent_sender: UnboundedSender<AgentActions>,
    mut gui_receiver: UnboundedReceiver<GuiEvents>,
    directory: String,
    file: String,
) {
    let mut app = App::new(config, agent_sender, directory, file);

    let mut reader = EventStream::new();

    loop {
        tokio::select! {
            Some(event) = gui_receiver.recv() => {
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
                            // handle key press
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
        }
    }
}

fn ui(frame: &mut Frame, app: &App) {
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
}
