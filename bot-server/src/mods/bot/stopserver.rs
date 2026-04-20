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
    println!(
        "Sending stop signal to socket '{}'",
        &interaction.channel_id.get()
    );

    let agent = appstate.find_connection_by_guild(get_guild(ctx, interaction).await?)?;
    let response = CreateInteractionResponseMessage::new();
    if let Err(e) = agent.stop_server().await {
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

pub fn register() -> CreateCommand {
    CreateCommand::new("stopserver").description("Stop your minecraft server")
}
