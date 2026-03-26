use crate::appstate::AppState;
use anyhow::{Result, anyhow};
use protocol::agentactions::AgentActions;
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

pub async fn start_chat(
    interaction: &CommandInteraction,
    appstate: &AppState,
    client: Arc<Client>,
) -> Result<()> {
    let id = interaction
        .guild_id
        .ok_or_else(|| anyhow!("Interaction happened outside of guild"))?
        .get();
    let agent = appstate.find_connection_by_guild(id)?;
    agent.start_chat_loop(client.clone()).await?;
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("startchat").description("Start your minecraft server")
}
