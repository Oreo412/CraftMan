use std::env;

use serenity::prelude::*;

use crate::bot_handler;
use crate::mods::appstate;

pub async fn start_bot(appstate: appstate::AppState) {
    // Configure the client with your Discord bot token in the environment.
    dotenvy::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(token.clone(), GatewayIntents::empty())
        .event_handler(bot_handler::Handler {
            twilight_client: appstate.twilight_client.clone(), //Creates a Twilight HTTP client. Serenity Client is made first so Token needs to be cloned. Then Token is moved into Twilight Client, consuming it.
            app_state: appstate,
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
