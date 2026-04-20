use crate::appstate::AppState;
use crate::mods::bot::get_guild::get_guild;
use anyhow::Result;
use serenity::all::Context;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;

pub async fn stop_chat(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: &AppState,
) -> Result<()> {
    let id = get_guild(ctx, interaction).await?;
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
