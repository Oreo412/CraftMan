use crate::appstate::AppState;
use anyhow::Result;
use anyhow::anyhow;
use protocol::agentactions::AgentActions;
use serenity::all::Context;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use serenity::model::application::*;
use uuid::Uuid;

pub async fn stop_minecraft_server(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: &AppState,
) -> Result<()> {
    println!(
        "Sending stop signal to socket '{}'",
        &interaction.channel_id.get()
    );

    let agent = appstate.find_connection_by_guild(
        interaction
            .guild_id
            .ok_or_else(|| anyhow!("interaction outside of guild"))?
            .get(),
    )?;
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
