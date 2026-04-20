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
use tracing::{debug, error, instrument, warn};

#[instrument(skip(receiver, agent, twilight_client))]
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
                error!("Error handling received action: {}", e);
            }
        } else {
            warn!("Received unhandled message in websocket.");
        }
    }
    println!("Connection lost!");
    agent.lost_connection().await;
}

#[instrument(skip(agent, twilight_client))]
async fn handle_message(
    message: ServerActions,
    agent: Arc<Agent>,
    twilight_client: Arc<twilight_http::Client>,
) -> Result<()> {
    println!("Handling message");
    match message {
        ServerActions::PropsResponse(id, props) => {
            debug!("Handling props response");
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
            debug!("Handling query response");
            agent
                .complete_request(
                    &id,
                    RequestResponses::QueryResponse(description, image, status),
                )
                .await?;
        }
        ServerActions::UpdateQuery { status } => {
            debug!("Handling update query");
            if let Some((channel_id, message_id)) = agent.query_ids().await? {
                update_monitor(channel_id, message_id, status, &twilight_client).await?;
            } else {
                warn!("Update query received without an associated monitor");
                agent.send(AgentActions::StopQuery).await?;
            }
        }
        ServerActions::UpdateQueryHeader { description, image } => {
            debug!("Handling update query header");
            if let Some((channel_id, message_id)) = agent.query_ids().await? {
                update_header(message_id, channel_id, description, image, &twilight_client).await?;
            } else {
                debug!("Update query header received without an associated monitor");
                agent.send(AgentActions::StopQuery).await?;
            }
        }
        ServerActions::NewMessage(message) => {
            warn!("New message, a testing enum received");
            agent.send_chat(message).await?;
        }
        ServerActions::StartResponse(id) => {
            debug!("Handling start response");
            agent
                .complete_request(&id, RequestResponses::StartServerResponse)
                .await?;
        }
        ServerActions::StopResponse(id) => {
            debug!("Handling stop response");
            agent
                .complete_request(&id, RequestResponses::StopServerResponse)
                .await?;
        }
        ServerActions::ConnectAgent(_) => {
            bail!("Agent already connected")
        }
        ServerActions::StartChatResponse(id) => {
            debug!("Handling start chat response");
            agent
                .complete_request(&id, RequestResponses::StartChatResponse)
                .await?;
        }
        ServerActions::StopChatResponse(id) => {
            debug!("Handling stop chat response");
            agent
                .complete_request(&id, RequestResponses::StopChatResponses)
                .await?;
        }
        ServerActions::SendCommandResponse(id) => {
            debug!("Handling send command response");
            agent
                .complete_request(&id, RequestResponses::CommandResponse)
                .await?;
        }
    }
    Ok(())
}
