use crate::appstate::AppState;
use anyhow::Result;
use axum::extract::ws::Message;
use protocol::agentactions::AgentActions;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use serenity::model::application::*;
use serenity::prelude::Context;
use std::error::Error;
use uuid::Uuid;

pub async fn start_mc_server(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: &AppState,
) -> Result<()> {
    let id = interaction
        .data
        .options
        .iter()
        .find(|option| option.name == "name")
        .unwrap();

    println!(
        "Sending stop signal to socket '{}'",
        &id.value.as_str().unwrap()
    );
    if let Err(e) = appstate
        .send_message(
            id.value.as_str().unwrap().to_string(),
            AgentActions::SvStop(Uuid::new_v4()),
        )
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
