use crate::mods::{configs::Configs, server_handler::ServerHandler, *};
use anyhow::Result;
use futures_util::{
    sink::{Sink, SinkExt},
    stream::{SplitStream, StreamExt},
};
use protocol::serveractions::ServerActions;
use std::env;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{WebSocket, protocol::Message},
};

pub async fn connect(handler: &mut ServerHandler) -> anyhow::Result<()> {
    let url = env::var("URL")?;

    let (ws_stream, _) = connect_async(url).await?;

    let (ws_write, mut ws_read) = ws_stream.split();

    let (sender, receiver) = mpsc::unbounded_channel();

    tokio::spawn(send_task(receiver, ws_write));

    listener::listen(ws_read, sender, handler).await?;

    Ok(())
}

async fn send_task<S>(mut receiver: UnboundedReceiver<ServerActions>, mut sender: S) -> Result<()>
where
    S: Sink<Message> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    while let Some(message) = receiver.recv().await {
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
