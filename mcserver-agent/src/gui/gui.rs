use crate::{gui::app::App, mods::configs::Configs};
use crossterm::event::{self, Event};
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

pub fn build_handler(directory: String, file: String) -> UnboundedSender<GuiEvents> {
    let (gui_sender, gui_receiver) = mpsc::unbounded_channel::<GuiEvents>();
    let tokio::spawn(todo!());
    gui_sender
}

pub enum GuiEvents {
    Validate(String),
    Validated,
    ServerStarted,
    ServerStopped,
}

async fn handler(
    config: &Configs,
    agent_sender: UnboundedSender<AgentActions>,
    gui_receiver: UnboundedReceiver<GuiEvents>,
    directory: String,
    file: String,
) {
    let mut app = App::new(config, agent_sender, directory, file);
    loop {
        tokio::select! {
            Some(event) = gui_receiver.recv() => {},
            Event::Key(key) = event::read()? => {}
        }
    }
}

fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let status = Paragraph::new("Server is running").block(
        Block::default()
            .title("Craftman Agent")
            .borders(Borders::ALL),
    );

    frame.render_widget(status, chunks[0]);

    let config_text = vec![
        Span::from(format!("Directory: {}", app.config.dir)),
        Span::from(format!("Min Memory: {}", app.config.xms)),
        Span::from(format!("Max Memory: {}", app.config.xmx)),
        Span::from(format!("Server Jar: {}", app.config.jar)),
    ];

    let 
}
