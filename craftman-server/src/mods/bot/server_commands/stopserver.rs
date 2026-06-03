use crate::appstate::AppState;
use crate::mods::bot::get_guild::get_guild;
use anyhow::Result;
use serenity::all::Context;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;

pub async fn stop_minecraft_server(
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
    let response = CreateInteractionResponseMessage::new();
    if let Err(e) = agent.stop_server().await {
        tracing::warn!("Stop server failed: {}", e);
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(response.content(e.to_string())),
            )
            .await?;
    } else {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(response.content("Successfully stopped server")),
            )
            .await?;
    }
    Ok(())
}
