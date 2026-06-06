use std::{io, time::Duration};

use crate::{
    gui::{
        app::{App, AppState, EditMemory, EditMemoryState},
        gui_actions::ConfigRequest,
    },
    mods::configs::Configs,
};
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::{FutureExt, StreamExt};
use ratatui::{
    Frame, Terminal,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, FrameExt, Paragraph},
};

use anyhow::Result;
use ratatui_explorer::{FileExplorerBuilder, Theme};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::interval,
};

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
        let mut config: Option<Configs> = None;
        tokio::select! {
                    Some(event) = tui_from_agent.recv() => {
                        match event {
                            GuiEvents::AddStdoutLine(line) => app.stdout.push_back(line),
                            GuiEvents::Validate(key) => app.start_validation(key),
                            GuiEvents::Validated => app.complete_validation(),
                            GuiEvents::ServerStarted => app.server_running = true,
                            GuiEvents::ServerStopped => app.server_running = false,
                        }

                    }

                    maybe_event = reader.next().fuse() => {
            match maybe_event {
                Some(Ok(event)) => {
                    if let Event::Key(key) = &event &&
                        key.kind == KeyEventKind::Press {
                            let code = key.code;

                            match &mut app.state {
                                AppState::Default => match code {
                                    KeyCode::Char('q') => {
                                        app.state = AppState::Exiting;
                                    }
                                    KeyCode::Char('c') => {
                                        let theme = Theme::default()
                                                    .add_default_title()
                                                    .with_block(Block::default().borders(Borders::ALL))
                                                    .with_highlight_item_style(Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD),)
                                                    .with_highlight_dir_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),)
                                                    .with_highlight_symbol(">> ");

                                        let mut explorer = match FileExplorerBuilder::default().working_dir(app.config.dir.clone()).build() {
                                            Err(e) => {

                                            tracing::error!("Error building file explorer: {}", e);
                                            break;

                                            }
                                            Ok(explorer) => {
                                                explorer
                                            }
                                        };
                                        explorer.set_theme(theme);
                                        app.state = AppState::FileSelection(explorer);
                                    }
                                    KeyCode::Char('x') => {
                                        app.state = AppState::EditMemory(EditMemory::new());
                                    }
                                    _ => {}
                                },

                                AppState::FileSelection(explorer) => match code {
                                    KeyCode::Enter => {
                                        app.state = AppState::FileSelectionConfirm(explorer.clone());
                                    }
                                    KeyCode::Esc => {
                                        app.state = AppState::Default;
                                    }
                                    _ => {
                                        explorer.handle(&event)?;
                                    }
                                },
                                AppState::FileSelectionConfirm(explorer) => match code {
                                    KeyCode::Enter => {
                                        tracing::info!("File selected. Exiting file_selection");
                                        let current = explorer.current();
                                        config = Some(
                                            app.config
                                                .clone()
                                                .set_jar(current.name.to_string())
                                                .set_dir(explorer.cwd().display().to_string()),
                                        );

                                        app.state = AppState::Default;
                                    }
                                    KeyCode::Esc => {
                                        app.state = AppState::FileSelection(explorer.clone())
                                    }
                                    _ => {}
                                }
                                AppState::Validate(_) => {
                                    if let KeyCode::Char('q') = code {
                                        break;
                                    }
                                }

                                AppState::EditMemory(current) => {
                                if current.state != EditMemoryState::IsThisCorrect {
                                    let editing = if current.state == EditMemoryState::Editxms {
                                        &mut current.xms_string
                                    } else {
                                        &mut current.xmx_string
                                    };
                                    match key.code {
                                    KeyCode::Esc => {app.state = AppState::Default;}
                                    KeyCode::Enter => {
                                        if  current.verify().is_err() {
                                            current.invalid_input = true;
                                        } else {
                                            current.invalid_input = false;
                                            current.state = EditMemoryState::IsThisCorrect;
                                        }
                                    }
                                    KeyCode::Left | KeyCode::Right => {
                                        if current.state == EditMemoryState::Editxms {
                                            current.state = EditMemoryState::Editxmx;
                                        } else if current.state == EditMemoryState::Editxmx {
                                            current.state = EditMemoryState::Editxms;
                                        }
                                    }
                                    KeyCode::Char(c) if c.is_numeric() || c == 'G' || c == 'g' => {
                                        editing.push(c.to_ascii_uppercase());
                                    }
                                    _ => {}
                                }} else if key.code == KeyCode::Enter {
                                        let new_config = app.config.clone().set_xms(current.xms.unwrap()).set_xmx(current.xmx.unwrap());
                                        if let Err(e) = app.edit_config(new_config).await {
                                            tracing::error!("Error editing config: {}", e);
                                        }
                                        app.state = AppState::Default;
                                    } else {
                                        current.state = EditMemoryState::Editxms;

                                }
                            }
                            AppState::Exiting => {
                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Enter => {break;},
                                    KeyCode::Esc => {app.state = AppState::Default}
                                    _ => {}
                                }
                            }
                                _ => {}
                            }
                        }

                }

                Some(Err(e)) => {
                    tracing::error!("TUI error has occured: {}", e);
                }

                None => break,
            }
        }
                    _ = tick.tick() => {
                        terminal.draw(|f| ui(f, &app))?;
                    }
                }
        if let Some(config) = config
            && let Err(e) = app.edit_config(config).await
        {
            tracing::error!("Failed to edit config: {}", e);
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(frame: &mut Frame, app: &App) {
    frame.render_widget(Clear, frame.area());
    if let AppState::FileSelection(explorer) | AppState::FileSelectionConfirm(explorer) = &app.state
    {
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

        if let AppState::FileSelectionConfirm(_) = &app.state {
            let area = frame.area();

            let popup_area = area.centered(Constraint::Percentage(60), Constraint::Percentage(30));

            let popup = Paragraph::new(format!(
                "Select this file?\n{}/{}",
                explorer.cwd().display(),
                explorer.current().name
            ))
            .block(Block::default().title("Popup").borders(Borders::ALL));

            frame.render_widget(Clear, popup_area);
            frame.render_widget(popup, popup_area);
        }

        return;
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let status = Paragraph::new(if app.server_running {
        "Server is running".green()
    } else {
        "Server is off".red()
    })
    .block(
        Block::default()
            .title("Craftman Agent")
            .borders(Borders::ALL),
    );

    frame.render_widget(status, chunks[0]);

    let config_text: Vec<Line<'_>> = [
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

    let block = Block::default()
        .title("Server Output")
        .borders(Borders::ALL);

    let scroll = stdout_lines
        .len()
        .saturating_sub(block.inner(chunks[2]).height as usize) as u16;

    let stdout = Paragraph::new(stdout_lines)
        .block(block)
        .scroll((scroll, 0));

    frame.render_widget(stdout, chunks[2]);

    let keys = Paragraph::new("q: quit | c: change server file | x: edit min and max memory")
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

    if let AppState::EditMemory(current) = &app.state {
        edit_memory(frame, current);
    }

    if let AppState::Exiting = &app.state {
        let area = frame.area();

        let popup_area = area.centered(Constraint::Percentage(60), Constraint::Percentage(30));

        let popup = Paragraph::new("Are you sure you would like to exit?\nPress enter or q to exit. Press escape to go back")
            .block(Block::default().title("Popup").borders(Borders::ALL));

        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup, popup_area);
    }
}

fn edit_memory(frame: &mut Frame, current: &EditMemory) {
    let area = frame.area();

    let popup_area = area.centered(Constraint::Percentage(60), Constraint::Percentage(30));

    frame.render_widget(Clear, popup_area);

    let outer = Block::default().title("Edit values").borders(Borders::ALL);

    frame.render_widget(outer, popup_area);

    let inner = popup_area.inner(Margin {
        vertical: 2,
        horizontal: 2,
    });

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(inner);

    let content_area = vertical_chunks[0];
    let guide_area = vertical_chunks[1];

    let mut lines = vec![
        Line::from("Enter minimum and maximum amount of memory given to the server"),
        Line::from("Arrow keys: Move between boxes | Enter: Set minimum and maximum | Esc: Exit"),
    ];

    if current.invalid_input {
        lines.push(Line::from("Please enter a valid input".red()));
    }
    let guide = Paragraph::new("Enter minimum and maximum amount of memory given to the server")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    if let EditMemoryState::IsThisCorrect = current.state {
        let is_this_correct = Paragraph::new(format!(
            "Is this correct?\nxms: {}\nxmx: {}",
            current.xms_string, current.xmx_string
        ));

        frame.render_widget(is_this_correct, content_area);
        frame.render_widget(guide, guide_area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .spacing(1)
        .split(content_area);

    let left_border = if current.state == EditMemoryState::Editxms {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let right_border = if current.state == EditMemoryState::Editxmx {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let left_box = Paragraph::new(current.xms_string.as_str()).block(
        Block::default()
            .title("Minimum")
            .borders(Borders::ALL)
            .border_style(left_border),
    );

    let right_box = Paragraph::new(current.xmx_string.as_str()).block(
        Block::default()
            .title("Maximum")
            .borders(Borders::ALL)
            .border_style(right_border),
    );

    frame.render_widget(left_box, chunks[0]);
    frame.render_widget(right_box, chunks[1]);
    frame.render_widget(guide, guide_area);
}
