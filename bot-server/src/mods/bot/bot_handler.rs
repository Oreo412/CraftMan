use crate::bot::*;
use crate::mods::*;
use serenity::async_trait;
use serenity::builder::{
    CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::{Command, Interaction, ResolvedOption};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use std::env;

pub struct Handler {
    pub app_state: crate::appstate::AppState,
    pub twilight_client: twilight_http::Client,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                println!("Received command interaction: {command:#?}");
                let _result = match command.data.name.as_str() {
                    "send_ws" => {
                        crate::bot::send_ws::run(&ctx, &command, self.app_state.clone()).await
                    }
                    "startserver" => {
                        crate::bot::startserver::start_mc_server(
                            &ctx,
                            &command,
                            self.app_state.clone(),
                        )
                        .await
                    }
                    "stopserver" => {
                        crate::bot::stopserver::start_mc_server(
                            &ctx,
                            &command,
                            self.app_state.clone(),
                        )
                        .await
                    }
                    "twilighttest" => {
                        crate::bot::settingsview::run(&self.twilight_client, command).await
                    }
                    _ => {
                        command
                            .create_response(ctx.http, CreateInteractionResponse::Acknowledge)
                            .await
                    }
                };
            }
            Interaction::Component(component) => match component.data.custom_id.as_str() {
                _ => {
                    let _response = component
                        .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
                        .await;
                }
            },
            Interaction::Modal(modal) => match modal.data.custom_id.as_str() {
                _ => {
                    let _response = modal
                        .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
                        .await;
                }
            },
            _ => println!("uh oh..."),
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = guild_id
            .set_commands(
                &ctx.http,
                vec![
                    send_ws::register(),
                    startserver::register(),
                    stopserver::register(),
                    settingsview::register(),
                ],
            )
            .await;

        println!("I now have the following guild slash commands: {commands:#?}");
    }
}
