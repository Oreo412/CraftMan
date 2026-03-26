use crate::appstate::AppState;
use anyhow::Result;
use anyhow::anyhow;
use protocol::agentactions::AgentActions;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use serenity::model::application::*;
use uuid::Uuid;

pub async fn start_mc_server(interaction: &CommandInteraction, appstate: &AppState) -> Result<()> {
    println!(
        "Sending start signal to socket '{}'",
        &interaction.channel_id.get()
    );
    if let Err(e) = appstate
        .send_by_guild(
            interaction
                .guild_id
                .ok_or_else(|| anyhow!("Interaction took place outside a guild"))?
                .get(),
            AgentActions::SvStart(Uuid::new_v4()),
        )
        .await
    {
        println!("Error sending to agent: {}", e);
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    let id = CreateCommandOption::new(CommandOptionType::String, "name", "Name of socket");
    CreateCommand::new("startserver")
        .description("Start your minecraft server")
        .add_option(id)
}
