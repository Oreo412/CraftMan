use crate::appstate::AppState;
use crate::mods::bot::get_guild::get_guild;
use anyhow::Result;
use serenity::all::Context;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use std::sync::Arc;
use twilight_http::Client;

pub async fn start_chat(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: &AppState,
    client: Arc<Client>,
) -> Result<()> {
    let id = get_guild(ctx, interaction).await?;
    let agent = appstate.find_connection_by_guild(id)?;
    let response = CreateInteractionResponseMessage::new();
    if let Err(e) = agent.start_chat_loop(client.clone()).await {
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
                    response.content("Successfully started chat loop"),
                ),
            )
            .await?;
    }
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("startchat").description("Forward chat from minecraft to discord")
}
