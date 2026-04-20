use crate::{appstate::AppState, mods::bot::get_guild::get_guild};
use anyhow::{Result, anyhow};
use serenity::all::Context;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use tracing::{info_span, warn};

pub async fn set_chat_channel(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: &AppState,
) -> Result<()> {
    let id = get_guild(ctx, interaction).await?;
    let agent = appstate.find_connection_by_guild(id)?;
    let span = info_span!("bot request for agent", agent_id = %agent.id());
    let _entered = span.enter();
    let response = CreateInteractionResponseMessage::new();
    if let Err(e) = agent.set_chat_channel(interaction.channel_id.get()).await {
        warn!("Request failed");
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
