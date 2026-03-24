use crate::appstate::AppState;
use anyhow::Result;
use axum::extract::ws::Message;
use protocol::agentactions::AgentActions;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use serenity::model::application::*;
use serenity::prelude::Context;
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::interval;
use tokio_tungstenite::tungstenite::buffer;
use twilight_http::Client;
use twilight_model::channel::ChannelType;
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;

pub async fn set_chat_channel(
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
        "Sending channel to socket '{}'",
        &id.value.as_str().unwrap()
    );
    if let Err(e) = appstate
        .send_message(
            id.value.as_str().unwrap().to_string(),
            AgentActions::SetChatChannel(interaction.channel_id.get()),
        )
        .await
    {
        println!("Error sending message via websocket: {}", e);
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    let id = CreateCommandOption::new(CommandOptionType::String, "name", "Name of socket");
    CreateCommand::new("set_chat")
        .description("Set this channel as your minecraft server chat")
        .add_option(id)
}

pub async fn chat_loop(
    channel_id: u64,
    mut receiver: UnboundedReceiver<String>,
    client: Client,
) -> Result<()> {
    let mut interval = interval(Duration::from_secs(2));
    let mut buffer: Vec<String> = Vec::new();
    let channel: Id<ChannelMarker> = Id::new(channel_id);
    loop {
        tokio::select! {
            message = receiver.recv() => {
                match message {
                    Some(text) => {
                        buffer.push(text);
                    }
                    None => break
                }
            }
            _ = interval.tick() => {
                if !buffer.is_empty() {
                    client.create_message(channel).content(&buffer.join("\n")).await?;
                    buffer.clear();
                }
            }
        }
    }
    Ok(())
}
