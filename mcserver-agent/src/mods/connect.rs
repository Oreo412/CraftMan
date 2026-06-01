use crate::gui::gui::GuiEvents;
use crate::gui::gui_actions::ConfigRequest;
use crate::mods::{configs::Configs, server_handler::ServerHandler, *};
use anyhow::Result;
use futures_util::{
    sink::{Sink, SinkExt},
    stream::{SplitStream, StreamExt},
};
use protocol::serveractions::ServerActions;
use std::env;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{WebSocket, protocol::Message},
};

pub const URL: &str = match option_env!("AGENT_DOMAIN") {
    Some(domain) => domain,
    None => "localhost:3000",
};

pub async fn connect(
    handler: &mut ServerHandler,
    agent_from_tui: &mut UnboundedReceiver<ConfigRequest>,
    agent_to_tui: UnboundedSender<GuiEvents>,
) -> anyhow::Result<()> {
    let url = format!("ws://{}/craftman", URL);

    let (ws_stream, _) = connect_async(url).await?;

    let (ws_write, ws_read) = ws_stream.split();

    let (sender, receiver) = mpsc::unbounded_channel();

    tokio::spawn(send_task(receiver, ws_write));

    sender.send(ServerActions::ConnectAgent(handler.id()))?;
    tracing::info!("Connected to server!");

    listener::listen(
        ws_read,
        sender,
        handler,
        agent_from_tui,
        agent_to_tui.clone(),
    )
    .await?;

    Ok(())
}

async fn send_task<S>(mut receiver: UnboundedReceiver<ServerActions>, mut sender: S) -> Result<()>
where
    S: Sink<Message> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    while let Some(message) = receiver.recv().await {
        tracing::info!("Sending server action");
        sender
            .send(Message::Text(
                serde_json::to_string(&message)
                    .expect("send_task serialization failed. This should not be possible. Major programming bug")
                    .into(),
            ))
            .await?;
    }
    Ok(())
}
