use crate::appstate::AppState;
use axum::extract::ws::Message;
use protocol::agentactions::AgentActions;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use serenity::model::application::*;
use serenity::prelude::Context;
use std::error::Error;

pub async fn start_mc_server(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: &AppState,
) -> Result<(), Box<dyn Error>> {
    let id = interaction
        .data
        .options
        .iter()
        .find(|option| option.name == "name")
        .unwrap();

    println!(
        "Sending start signal to socket '{}'",
        &id.value.as_str().unwrap()
    );
    if let Err(e) = appstate
        .send_message(
            id.value.as_str().unwrap().to_string(),
            AgentActions::sv_start,
        )
        .await
    {
        println!("Error sending message via websocket: {}", e);
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    let id = CreateCommandOption::new(CommandOptionType::String, "name", "Name of socket");
    CreateCommand::new("startserver")
        .description("Start your minecraft server")
        .add_option(id)
}
