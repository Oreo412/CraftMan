use crate::mods::agents::Agent;
use crate::mods::bot::chat_channel;
use crate::mods::bot::create_monitor::{update_header, update_monitor};
use anyhow::Result;
use axum::Error;
use axum::extract::ws::Message;
use futures_util::Stream;
use futures_util::stream::StreamExt;
use protocol::agentactions::AgentActions;
use protocol::serveractions::{OneshotResponses, ServerActions};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedSender, WeakUnboundedSender, unbounded_channel};

pub async fn listen<R>(
    mut receiver: R,
    agent: Arc<Agent>,
    twilight_client: Arc<twilight_http::Client>,
) where
    R: Stream<Item = Result<Message, Error>> + Unpin,
{
    let mut chat_channel: Option<WeakUnboundedSender<String>> = None;
    while let Some(msg) = receiver.next().await {
        if let Message::Text(text) = msg.unwrap()
            && let Ok(message) = serde_json::from_str::<ServerActions>(text.as_str())
        {
            if let Err(e) = handle_message(message, agent.clone(), twilight_client.clone()).await {
                println!("Error handling received action: {}", e);
            }
        } else {
            println!("Received unhandled message in websocket.");
        }
    }
}

async fn handle_message(
    message: ServerActions,
    agent: Arc<Agent>,
    twilight_client: Arc<twilight_http::Client>,
) -> Result<()> {
    match message {
        ServerActions::PropsResponse(id, props) => {
            agent.complete_request(&id, OneshotResponses::PropsResponse(props))?;
            // if let Some((_id, sender)) = something {
            //     if sender.send(OneshotResponses::PropsResponse(props)).is_err() {
            //         println!("Error sending properties to pending request: {}", id);
            //     } else {
            //         println!("Properties sent to pending request successfully");
            //     }
            // } else {
            //     println!("No pending request found for ID: {}", id);
            // }
        }
        ServerActions::QueryResponse {
            uuid: id,
            description,
            image,
            status,
        } => {
            agent.complete_request(
                &id,
                OneshotResponses::QueryResponse(description, image, status),
            )?;
        }
        ServerActions::UpdateQuery {
            message_id,
            channel_id,
            status,
        } => {
            update_monitor(message_id, channel_id, status, &twilight_client).await?;
        }
        ServerActions::UpdateQueryHeader {
            message_id,
            channel_id,
            description,
            image,
        } => {
            println!("Updating query header");
            update_header(message_id, channel_id, description, image, &twilight_client).await?;
        }
        ServerActions::NewMessage(message) => {
            agent.send_chat(message).await?;
        }

        _ => {
            println!("Received unhandled action:");
        }
    }
    Ok(())
}
