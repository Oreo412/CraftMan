use crate::appstate::AppState;
use anyhow::Result;
use protocol::agentactions::AgentActions;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use serenity::model::application::*;
use uuid::Uuid;

pub async fn start_mc_server(interaction: &CommandInteraction, appstate: &AppState) -> Result<()> {
    println!("Sending stop signal to socket '{}'", &interaction.id.get());
    if let Err(e) = appstate
        .send_by_guild(interaction.id.get(), AgentActions::SvStop(Uuid::new_v4()))
        .await
    {
        println!("Error sending message via websocket: {}", e);
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    let id = CreateCommandOption::new(CommandOptionType::String, "name", "Name of socket");
    CreateCommand::new("stopserver")
        .description("Stop your minecraft server")
        .add_option(id)
}
