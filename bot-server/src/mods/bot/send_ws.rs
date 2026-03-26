use anyhow::{Result, anyhow};
use appstate::AppState;
use protocol::agentactions::AgentActions;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use serenity::model::application::*;

use crate::mods::*;

pub async fn run(interaction: &CommandInteraction, appstate: AppState) -> Result<()> {
    let message = interaction
        .data
        .options
        .iter()
        .find(|option| option.name == "message")
        .unwrap();
    let id = interaction
        .guild_id
        .ok_or_else(|| anyhow!("Interaction took place outside of guild"))?
        .get();
    println!(
        "Sending message '{}' to socket '{}'",
        &message.value.as_str().unwrap(),
        id
    );
    if let Err(e) = appstate
        .send_by_guild(
            id,
            AgentActions::Message(message.value.as_str().unwrap().to_string()),
        )
        .await
    {
        println!("Error sending message via websocket: {}", e);
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    let id = CreateCommandOption::new(CommandOptionType::String, "name", "Name of socket");
    let message = CreateCommandOption::new(CommandOptionType::String, "message", "Message to send");
    CreateCommand::new("send_ws")
        .description("Send a message to a websocket")
        .add_option(id)
        .add_option(message)
}
