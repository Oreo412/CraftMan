mod gui;
mod mods;
use crate::{
    gui::{
        gui::{GuiEvents, handler},
        gui_actions::ConfigRequest,
    },
    mods::{server_handler::ServerHandler, stdout_writer::TuiWriter, *},
};
use connect::connect;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing_appender;
use tracing_subscriber::{
    filter::LevelFilter,
    fmt,
    layer::{Layer, SubscriberExt},
    util::SubscriberInitExt,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = configs::Configs::new();

    let (mut tui_to_agent, mut agent_from_tui) = mpsc::unbounded_channel::<ConfigRequest>();

    let (mut agent_to_tui, mut tui_from_agent) = mpsc::unbounded_channel::<GuiEvents>();

    let tui = tokio::spawn(handler(
        config.clone(),
        tui_to_agent.clone(),
        tui_from_agent,
    ));

    let mut handler = ServerHandler::new(config);

    let writer = TuiWriter::new(agent_to_tui.clone());

    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("craftman")
        .filename_suffix("log")
        .max_log_files(7)
        .build("logs")
        .expect("failed to initialize rolling file appender");

    let (file_writer, _log_guard) = tracing_appender::non_blocking(file_appender);

    let tui_layer = fmt::layer()
        .with_writer(move || writer.clone())
        .without_time()
        .with_target(false);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true);

    tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(tui_layer)
        .with(file_layer)
        .init();

    let backend = async {
        loop {
            match connect(&mut handler, &mut agent_from_tui, agent_to_tui.clone()).await {
                Ok(()) => {
                    tracing::info!("Disconnected. Reconnecting...");
                }
                Err(e) => {
                    tracing::info!("Connection failed: {e}");
                }
            }

            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    };

    tokio::select! {
        result = tui => {
            match result {
                Ok(_) => tracing::info!("TUI exited. Shutting down app."),
                Err(e) => tracing::error!("TUI task failed: {e}"),
            }
        }

        _ = backend => {
            tracing::info!("App task ended.");
        }
    }

    if let Err(e) = handler.stop_server().await {
        tracing::error!("Error shutting down server: {}", e)
    }
}
