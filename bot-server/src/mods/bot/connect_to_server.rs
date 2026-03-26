use crate::appstate::AppState;
use anyhow::Result;
use anyhow::anyhow;
use protocol::agentactions::AgentActions;
use serenity::all::Context;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use serenity::model::application::*;
use uuid::Uuid;

pub async fn connect_server(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: &AppState,
) -> Result<()> {
    let code = interaction.data.options[0]
        .value
        .as_str()
        .ok_or_else(|| anyhow!("Invalid code passed"))?;
    println!("Entered code: {}", code);
    let response = CreateInteractionResponseMessage::new().ephemeral(true);
    if let Err(e) = appstate
        .verify_agent(
            code,
            interaction
                .guild_id
                .ok_or_else(|| anyhow!("Interaction took place outside of guild"))?
                .get(),
        )
        .await
    {
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
                CreateInteractionResponse::Message(
                    response.content("Successfully verified connection"),
                ),
            )
            .await?;
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    let code = CreateCommandOption::new(CommandOptionType::String, "code", "Insert Connection Key");
    CreateCommand::new("verify")
        .description("Verify code")
        .add_option(code)
}
