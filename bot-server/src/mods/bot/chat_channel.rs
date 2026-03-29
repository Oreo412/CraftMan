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

pub async fn set_chat_channel(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: &AppState,
) -> Result<()> {
    let id = interaction
        .guild_id
        .ok_or_else(|| anyhow!("Interaction outside of guild"))?
        .get();

    let agent = appstate.find_connection_by_guild(id)?;
    let response = CreateInteractionResponseMessage::new();
    if let Err(e) = agent.set_chat_channel(interaction.channel_id.get()).await {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    response.content(format!("Error setting chat channel: {}", e)),
                ),
            )
            .await?
    } else {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    response.content("Successfully set chat channel"),
                ),
            )
            .await?;
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("set_chat").description("Set this channel as your minecraft server chat")
}
