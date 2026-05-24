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
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() {
    println!("starting");
    dotenvy::dotenv().ok();
    println!("poopy");
    let config = configs::Configs::new();

    println!("configs ready");

    let (mut tui_to_agent, mut agent_from_tui) = mpsc::unbounded_channel::<ConfigRequest>();

    let (mut agent_to_tui, mut tui_from_agent) = mpsc::unbounded_channel::<GuiEvents>();

    println!("Got to here");

    tokio::spawn(handler(
        config.clone(),
        tui_to_agent.clone(),
        tui_from_agent,
    ));

    println!("After spawn?");

    let mut handler = ServerHandler::new(config);

    let writer = TuiWriter::new(agent_to_tui.clone());

    tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .without_time()
        .with_target(false)
        .init();

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
}
