use crate::{appstate::AppState, mods::bot::get_guild::get_guild};
use anyhow::Result;
use serenity::all::Context;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use tracing::{info_span, warn};

pub async fn set_chat_channel(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: &AppState,
) -> Result<()> {
    let agent = if let Ok(agent) =
        appstate.find_connection_by_guild(get_guild(ctx, interaction).await?)
    {
        agent
    } else {
        let response = CreateInteractionResponseMessage::new();
        interaction.create_response(&ctx.http, CreateInteractionResponse::Message(response.content("Unable to find agent. Please either start agent or verify a new agent with /verify"))).await?;
        return Ok(());
    };
    let span = info_span!("bot request for agent", agent_id = %agent.id());
    let _entered = span.enter();
    let response = CreateInteractionResponseMessage::new();
    if let Err(e) = agent.set_chat_channel(interaction.channel_id.get()).await {
        warn!("Set Chat Channel Request failed");
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
