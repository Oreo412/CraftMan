mod mods;
use crate::mods::{server_handler::ServerHandler, *};
use anyhow::Result;
use directories::ProjectDirs;
use futures_util::{
    sink::{Sink, SinkExt},
    stream::{SplitSink, SplitStream, StreamExt},
};
use protocol::serveractions::ServerActions;
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::mpsc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{RwLock, mpsc::UnboundedReceiver},
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let url = env::var("URL").expect("Expected something in URL");
    let config = configs::Configs::new();

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    println!("Connected to server");

    let (write, mut read) = ws_stream.split();

    let (sender, receiver) = mpsc::unbounded_channel::<ServerActions>();

    tokio::spawn(send_task(receiver, write));
    if let Err(e) = sender.send(ServerActions::ConnectAgent(config.id)) {
        println!("Error sending initialization: {}", e);
    }

    let mut handler = ServerHandler::new(config);

    if let Err(e) = listener::listen(&mut read, sender, &mut handler).await {
        println!("Error: {}", e);
    }
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
