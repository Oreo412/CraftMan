use crate::appstate::AppState;
use crate::mods::bot::get_guild::get_guild;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use protocol::agentactions::AgentActions;
use protocol::server_commands::ServerCommands;
use serenity::all::Context;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use serenity::model::application::*;
use uuid::Uuid;

pub async fn send_to_minecraft(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: &AppState,
    command: &str,
) -> Result<()> {
    tracing::info!("Gonna send something to the minecraft server");
    let agent = appstate.find_connection_by_guild(get_guild(ctx, interaction).await?)?;
    let response = CreateInteractionResponseMessage::new();
    let command_data = if let CommandDataOptionValue::SubCommand(sub_options) =
        interaction.data.options[0].value.clone()
        && let CommandDataOptionValue::String(data) = sub_options[0].value.clone()
    {
        data
    } else {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(response.content("No command found")),
            )
            .await?;
        return Ok(());
    };
    tracing::info!("Sending following message: {}", command_data);
    let servercommand = match command {
        "say" => ServerCommands::Say(command_data.to_string()),
        "command" => ServerCommands::Command(command_data.to_string()),
        _ => {
            bail!("Something went wrong");
        }
    };
    if let Err(e) = agent.message_chat(servercommand).await {
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
                CreateInteractionResponse::Message(response.content("Sent command")),
            )
            .await?;
    }
    Ok(())
}
