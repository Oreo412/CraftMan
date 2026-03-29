use crate::appstate::AppState;
use anyhow::{Result, anyhow};
use protocol::agentactions::AgentActions;
use serenity::all::Context;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use serenity::model::application::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::interval;
use twilight_http::Client;
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;

pub async fn stop_chat(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: &AppState,
    client: Arc<Client>,
) -> Result<()> {
    let id = interaction
        .guild_id
        .ok_or_else(|| anyhow!("Interaction happened outside of guild"))?
        .get();
    let agent = appstate.find_connection_by_guild(id)?;
    let response = CreateInteractionResponseMessage::new();
    if let Err(e) = agent.stop_chat_stream().await {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    response.content(format!("Error starting chat loop: {}", e)),
                ),
            )
            .await?;
    } else {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    response.content("Successfully stopped chat loop"),
                ),
            )
            .await?;
    }
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("stopchat").description("Stop forwarding chat to discord")
}
