use futures_util::Stream;
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use protocol::agentactions::AgentActions;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::mods::svstarter;

pub async fn listen<S>(receiver: &mut S)
where
    S: Stream<Item = Result<Message, Error>> + Unpin,
{
    let mut process = svstarter::ServerProcess::default().dir(String::from("/home/oreo/mcserver/"));
    while let Some(msg) = receiver.next().await {
        if let Message::Text(text) = msg.unwrap() {
            if let Ok(message) = serde_json::from_str::<AgentActions>(text.as_str()) {
                match message {
                    AgentActions::message(content) => {
                        println!("Received message action: {}", content);
                    }
                    AgentActions::sv_start => {
                        if let Err(e) = process.start_server() {
                            println!("Error starting server: {}", e);
                        } else {
                            println!("Server started successfully");
                        }
                    }
                    AgentActions::sv_stop => {
                        if let Err(e) = process.stop_server() {
                            println!("Error stopping server: {}", e);
                        } else {
                            println!("Server stopped successfully");
                        }
                    }
                    _ => {
                        println!("Received unhandled action:");
                    }
                }
            }
        }
    }
}
