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
    while let Some(msg) = read.next().await {
        let msg = msg.unwrap();
        if let Message::Text(text) = msg {
            println!("Received: {}", text);
        }
    }
}

/*
fn main() {
    let child = svstarter::start_server();

    println!("Server running. Press Enter to stop...");
    let _result = std::io::stdin().read_line(&mut String::new());

    svstarter::stop_server(child.unwrap().stdin.unwrap());

    println!("Maybe the server stopped or something idk")
}
    */
