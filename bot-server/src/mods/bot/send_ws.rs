use appstate::AppState;
use axum::extract::ws::Message;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use serenity::model::application::*;
use serenity::prelude::Context;

use crate::mods::*;

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
    appstate: AppState,
) -> Result<(), serenity::Error> {
    let message = interaction
        .data
        .options
        .iter()
        .find(|option| option.name == "message")
        .unwrap();
    let id = interaction
        .data
        .options
        .iter()
        .find(|option| option.name == "name")
        .unwrap();

    println!(
        "Sending message '{}' to socket '{}'",
        &message.value.as_str().unwrap(),
        &id.value.as_str().unwrap()
    );
    if let Err(e) = appstate
        .send_message(
            id.value.as_str().unwrap().to_string(),
            message.value.as_str().unwrap().to_string(),
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
