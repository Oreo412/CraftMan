use crate::mods::agents::Agent;
use crate::mods::bot::chat_channel;
use crate::mods::bot::query_monitor::{update_header, update_monitor};
use anyhow::Result;
use anyhow::bail;
use axum::Error;
use axum::extract::ws::Message;
use futures_util::Stream;
use futures_util::stream::StreamExt;
use protocol::agentactions::AgentActions;
use protocol::serveractions::{RequestResponses, ServerActions};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedSender, WeakUnboundedSender, unbounded_channel};

pub async fn listen<R>(
    mut receiver: R,
    agent: Arc<Agent>,
    twilight_client: Arc<twilight_http::Client>,
) where
    R: Stream<Item = Result<Message, Error>> + Unpin,
{
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
            agent
                .complete_request(&id, RequestResponses::PropsResponse(props))
                .await?;
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
            println!("Handling query response");
            agent
                .complete_request(
                    &id,
                    RequestResponses::QueryResponse(description, image, status),
                )
                .await?;
        }
        ServerActions::UpdateQuery { status } => {
            if let Some((channel_id, message_id)) = agent.query_ids().await? {
                update_monitor(channel_id, message_id, status, &twilight_client).await?;
            } else {
                agent.send(AgentActions::StopQuery)?;
            }
        }
        ServerActions::UpdateQueryHeader { description, image } => {
            println!("Updating query header");
            if let Some((channel_id, message_id)) = agent.query_ids().await? {
                update_header(message_id, channel_id, description, image, &twilight_client).await?;
            } else {
                agent.send(AgentActions::StopQuery)?;
            }
        }
        ServerActions::NewMessage(message) => {
            agent.send_chat(message).await?;
        }
        ServerActions::StartResponse(id) => {
            println!("Received start response. Trying to complete request");
            agent
                .complete_request(&id, RequestResponses::StartServerResponse)
                .await?;
        }
        ServerActions::StopResponse(id) => {
            agent
                .complete_request(&id, RequestResponses::StopServerResponse)
                .await?;
        }
        ServerActions::ConnectAgent(_) => {
            bail!("Agent already connected")
        }
        ServerActions::StartChatResponse(id) => {
            println!("Received start chat response. Completing request");
            agent
                .complete_request(&id, RequestResponses::StartChatResponse)
                .await?;
        }
        ServerActions::StopChatResponse(id) => {
            println!("Received stop chat response. Completing request");
            agent
                .complete_request(&id, RequestResponses::StopChatResponses)
                .await?;
        }
        ServerActions::SendCommandResponse(id) => {
            agent
                .complete_request(&id, RequestResponses::CommandResponse)
                .await?;
        }
    }
    Ok(())
}
