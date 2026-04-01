use crate::mods::agents::Agent;
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

pub async fn listen<R>(
    mut receiver: R,
    agent: Arc<Agent>,
    twilight_client: Arc<twilight_http::Client>,
) where
    R: Stream<Item = Result<Message, Error>> + Unpin,
{
    while let Some(msg) = receiver.next().await {
        if let Ok(Message::Text(text)) = msg
            && let Ok(message) = serde_json::from_str::<ServerActions>(text.as_str())
        {
            if let Err(e) = handle_message(message, agent.clone(), twilight_client.clone()).await {
                println!("Error handling received action: {}", e);
            }
        } else {
            println!("Received unhandled message in websocket.");
        }
    }
    println!("Connection lost!");
    agent.lost_connection().await;
}

async fn handle_message(
    message: ServerActions,
    agent: Arc<Agent>,
    twilight_client: Arc<twilight_http::Client>,
) -> Result<()> {
    println!("Handling message");
    match message {
        ServerActions::PropsResponse(id, props) => {
            agent
                .complete_request(&id, RequestResponses::PropsResponse(props))
                .await?;
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
                agent.send(AgentActions::StopQuery).await?;
            }
        }
        ServerActions::UpdateQueryHeader { description, image } => {
            println!("Updating query header");
            if let Some((channel_id, message_id)) = agent.query_ids().await? {
                update_header(message_id, channel_id, description, image, &twilight_client).await?;
            } else {
                agent.send(AgentActions::StopQuery).await?;
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
