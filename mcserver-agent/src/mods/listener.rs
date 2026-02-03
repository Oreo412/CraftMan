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
    while let Some(msg) = receiver.next().await {
        if let Message::Text(text) = msg.unwrap() {
            if let Ok(message) = serde_json::from_str::<AgentActions>(text.as_str()) {
                match message {
                    AgentActions::message(content) => {
                        println!("Received message action: {}", content);
                    }
                    AgentActions::sv_start => {
                        if svstarter::start_server().is_err() {
                            println!("Error starting server");
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
