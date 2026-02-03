mod mods;

use std::{collections::HashMap, env, sync::Arc};

use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::mods::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let url = env::var("URL").expect("Expected something in URL");

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    let appstate = appstate::AppState::new();

    println!("Connected to server");

    let (mut write, mut read) = ws_stream.split();

    // Send hello message
    write.send(Message::Text("stinky".into())).await.unwrap();

    // Listen for messages
    listener::listen(&mut read).await;
}
