use crate::appstate::AppState;
use crate::mods::bot::get_guild::get_guild;
use anyhow::Result;
use serenity::all::Context;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;

pub async fn start_mc_server(
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
    if let Err(e) = agent.start_server().await {
        tracing::warn!("Start Server Failed: {}", e);
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(response.content(e.to_string())),
            )
            .await?;
        tracing::debug!("Is it getting to here???");
    } else {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(response.content("Successfully started server")),
            )
            .await?;
    }
    Ok(())
}
