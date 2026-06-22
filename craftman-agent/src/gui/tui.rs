use std::{io, time::Duration};

use crate::{
    gui::{
        app::{App, AppState, EditArgState, EditMemory, EditMemoryState},
        gui_actions::ConfigRequest,
    },
    mods::configs::{Configs, RunType},
};
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use futures::{FutureExt, StreamExt};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, FrameExt, List, ListDirection, ListItem, ListState, Paragraph,
    },
};

use anyhow::Result;
use ratatui_explorer::{FileExplorerBuilder, Theme};
use ratatui_textarea::{Input, TextArea};
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

    let mut terminal = ratatui::try_init()?;

    let mut app = App::new(config, tui_to_agent);

    terminal.clear()?;
    terminal.draw(|f| ui(f, &mut app))?;

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
                                    KeyCode::Char('a') => {
                                        if app.config.jar.ends_with(".sh") {
                                            app.state = AppState::CustomArgNotAllowed;
                                        } else {
                                            let args = if let RunType::CustomJar(ref args) = app.config.run_type {args.clone()} else { Vec::<String>::new()};
                                            app.state = AppState::EditArgs(args, ListState::default(), EditArgState::Default);
                                        }
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
                                AppState::EditArgs(args, list_state, edit_state) => {
                                    let mut key = *key;
                                    match handle_edit_arg(&mut key, args, list_state, edit_state) {
                                        ExitEditArgs::SaveAndExit(args) => {
                                            config = Some( app.config.clone().set_run_type(RunType::CustomJar(args)) );
                                            app.state = AppState::Default;
                                        }
                                        ExitEditArgs::ExitWithoutSave => {
                                            app.state = AppState::Default;
                                        }
                                        ExitEditArgs::Clear => {
                                            config = Some( app.config.clone().set_run_type(RunType::Default) );
                                            app.state = AppState::Default;
                                        }
                                        ExitEditArgs::None => {}
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
                            AppState::CustomArgNotAllowed => {
                                app.state = AppState::Default;
                            }
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
                        terminal.draw(|f| ui(f, &mut app))?;
                    }
                }
        if let Some(config) = config
            && let Err(e) = app.edit_config(config).await
        {
            tracing::error!("Failed to edit config: {}", e);
        }
    }

    ratatui::restore();

    Ok(())
}

fn handle_edit_arg(
    event: &mut KeyEvent,
    args: &mut Vec<String>,
    list_state: &mut ListState,
    edit_state: &mut EditArgState,
) -> ExitEditArgs {
    match edit_state {
        EditArgState::Default => match event.code {
            KeyCode::Esc => {
                *edit_state = EditArgState::ExitWithoutSaving;
            }
            KeyCode::Char('s') => {
                *edit_state = EditArgState::SaveAndExit;
            }
            KeyCode::Char('n') => {
                let mut arg = TextArea::new(Vec::<String>::new());
                arg.set_block(Block::bordered());
                *edit_state = EditArgState::Add(arg);
            }
            KeyCode::Char('e') => {
                if let Some(selected) = list_state.selected() {
                    let mut arg = TextArea::new(vec![args[selected].clone()]);
                    arg.set_block(Block::bordered());
                    *edit_state = EditArgState::Edit(arg, selected);
                }
            }
            KeyCode::Char('d') => {
                if let Some(selected) = list_state.selected() {
                    args.remove(selected);
                }
            }
            KeyCode::Char('c') => {
                *edit_state = EditArgState::Clear;
            }
            KeyCode::Down => {
                list_state.select_next();
            }
            KeyCode::Up => {
                list_state.select_previous();
            }
            _ => {}
        },
        EditArgState::Add(text) => match event.code {
            KeyCode::Esc => {
                *edit_state = EditArgState::Default;
            }
            KeyCode::Enter => {
                let line = text.lines().first();
                if let Some(arg) = line {
                    args.push(arg.clone());
                }
                *edit_state = EditArgState::Default;
            }
            _ => {
                text.input(Input::from(*event));
            }
        },
        EditArgState::Edit(text, location) => match event.code {
            KeyCode::Esc => {
                *edit_state = EditArgState::Default;
            }
            KeyCode::Enter => {
                let line = text.lines().first();
                if let Some(arg) = line {
                    args[*location] = arg.clone();
                }
                *edit_state = EditArgState::Default;
            }
            _ => {
                text.input(Input::from(*event));
            }
        },
        EditArgState::SaveAndExit => match event.code {
            KeyCode::Esc => {
                *edit_state = EditArgState::Default;
            }
            KeyCode::Enter => {
                return ExitEditArgs::SaveAndExit(args.clone());
            }
            _ => {}
        },
        EditArgState::ExitWithoutSaving => match event.code {
            KeyCode::Esc => {
                *edit_state = EditArgState::Default;
            }
            KeyCode::Enter => {
                return ExitEditArgs::ExitWithoutSave;
            }
            _ => {}
        },
        EditArgState::Clear => match event.code {
            KeyCode::Esc => {
                *edit_state = EditArgState::Default;
            }
            KeyCode::Enter => {
                return ExitEditArgs::Clear;
            }
            _ => {}
        },
    }
    ExitEditArgs::None
}

enum ExitEditArgs {
    SaveAndExit(Vec<String>),
    ExitWithoutSave,
    Clear,
    None,
}

fn ui(frame: &mut Frame, app: &mut App) {
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

    let run_type_text = match &app.config.run_type {
        RunType::Default => "Default".to_string(),
        RunType::Script => "Script".to_string(),
        RunType::CustomJar(args) => {
            format!("Custom: {}", args.join(" "))
        }
    };

    let config_text: Vec<Line<'_>> = [
        Span::from(format!("Directory: {}", app.config.dir)),
        Span::from(format!("Min Memory: {}", app.config.xms)),
        Span::from(format!("Max Memory: {}", app.config.xmx)),
        Span::from(format!("Server Jar: {}", app.config.jar)),
        Span::from(format!("Run Type: {}", run_type_text)),
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

    let keys = Paragraph::new(
        "q: quit | c: change server file | x: edit min and max memory | a: edit arguments (Advanced)",
    )
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
    if let AppState::EditArgs(args, list_state, edit_state) = &mut app.state {
        let area = frame.area();

        let popup_area = area.centered(Constraint::Percentage(80), Constraint::Percentage(50));
        frame.render_widget(Clear, popup_area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(4),
                Constraint::Length(5),
                Constraint::Length(3),
            ])
            .split(popup_area);

        if let EditArgState::SaveAndExit = edit_state {
            let paragraph =
                Paragraph::new("Save arguments?\nPress enter to save and escape to go back")
                    .alignment(Alignment::Center)
                    .block(Block::bordered());
            frame.render_widget(paragraph, chunks[0]);
            return;
        } else if let EditArgState::ExitWithoutSaving = edit_state {
            let paragraph = Paragraph::new(
                "Exit without saving arguments?\nPress enter to exit and escape to go back",
            )
            .alignment(Alignment::Center)
            .block(Block::bordered());
            frame.render_widget(paragraph, chunks[0]);
            return;
        } else if let EditArgState::Clear = edit_state {
            let paragraph = Paragraph::new(
                "Clear arguments and return to default?\nPress enter to confirm and escape to go back",
            ).centered().block(Block::bordered());
            frame.render_widget(paragraph, chunks[0]);
            return;
        }

        if let EditArgState::Edit(arg, _) | EditArgState::Add(arg) = edit_state {
            let area = frame.area();
            let popup_area = area.centered(Constraint::Percentage(60), Constraint::Length(3));
            frame.render_widget(Clear, popup_area);
            frame.render_widget_ref(&*arg, popup_area);
        }

        let items: Vec<ListItem> = args.iter().map(|s| ListItem::new(s.as_str())).collect();
        let list = List::new(items)
            .block(Block::bordered().title("Arguments"))
            .style(Style::new().white())
            .highlight_style(Style::new().italic())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);
        frame.render_stateful_widget(list, chunks[0], list_state);
        let guide = Paragraph::new("Please enter all arguments listed after Java when you run the server\nEnter -Xms and -Xmx to automatically use configured min and max\nIncluding @user_jvm_args.txt will automatically update script with configured minimum and maximum memory").alignment(Alignment::Center).block(Block::bordered());
        frame.render_widget(guide, chunks[1]);
        let keys = Paragraph::new(
            "esc: Exit without saving | n: new argument | e: edit argument | d: delete argument | s: save arguments | c: clear arguments (return to default)",
        )
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(keys, chunks[2]);
    }

    if let AppState::CustomArgNotAllowed = &app.state {
        let area = frame.area();

        let popup_area = area.centered(Constraint::Percentage(40), Constraint::Percentage(20));

        frame.render_widget(Clear, popup_area);

        let text = Paragraph::new("Custom arguments are not allowed for running a .sh script\nPress any key to exit popup").centered().block(Block::bordered());

        frame.render_widget(text, popup_area);
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
