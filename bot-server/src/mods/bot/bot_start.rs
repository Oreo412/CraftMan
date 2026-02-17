use std::env;

use serenity::async_trait;
use serenity::builder::{
    CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::{Command, Interaction, ResolvedOption};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use twilight_http::Client as TwilightClient;

use crate::bot_handler;
use crate::mods::appstate;
use crate::mods::bot::settingscreen;

pub async fn start_bot(appstate: appstate::AppState) {
    // Configure the client with your Discord bot token in the environment.
    dotenvy::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(token.clone(), GatewayIntents::empty())
        .event_handler(bot_handler::Handler {
            app_state: appstate,
            twilight_client: TwilightClient::new(token), //Creates a Twilight HTTP client. Serenity Client is made first so Token needs to be cloned. Then Token is moved into Twilight Client, consuming it.
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
