use crate::mods::agents::Agent;
use axum::Error;
use axum::extract::ws::Message;
use futures_util::Stream;
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use protocol::query_options::QueryStatus;
use protocol::serveractions::{OneshotResponses, ServerActions};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};
use tokio_tungstenite::connect_async;
use uuid::Uuid;

pub async fn listen<R>(mut receiver: R, agent: Arc<Agent>)
where
    R: Stream<Item = Result<Message, Error>> + Unpin,
{
    while let Some(msg) = receiver.next().await {
        if let Message::Text(text) = msg.unwrap() {
            if let Ok(message) = serde_json::from_str::<ServerActions>(text.as_str()) {
                match message {
                    ServerActions::PropsResponse(id, props) => {
                        let something = agent.pending_requests.remove(&id);
                        if let Some((_id, sender)) = something {
                            if let Err(e) = sender.send(OneshotResponses::PropsResponse(props)) {
                                println!("Error sending properties to pending request: {}", id);
                            } else {
                                println!("Properties sent to pending request successfully");
                            }
                        } else {
                            println!("No pending request found for ID: {}", id);
                        }
                    }
                    ServerActions::QueryResponse(id, description, image, query) => {
                        let something = agent.pending_requests.remove(&id);
                        if let Some((_id, sender)) = something {
                            if let Err(e) = sender.send(OneshotResponses::QueryResponse(
                                description,
                                image,
                                query,
                            )) {
                                println!("Error sending queryresponse to pending request: {}", id);
                            } else {
                                println!("response sent to pending request successfully");
                            }
                        } else {
                            println!("No pending request found for ID: {}", id);
                        }
                    }
                    ServerActions::UpdateQuery(message_id, channel_id, status)
                    _ => {
                        println!("Received unhandled action:");
                    }
                }
            }
        }
    }
}
