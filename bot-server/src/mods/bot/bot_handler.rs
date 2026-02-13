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
                        crate::bot::startserver::start_mc_server(&ctx, &command, &self.app_state)
                            .await
                    }
                    "stopserver" => {
                        crate::bot::stopserver::start_mc_server(&ctx, &command, &self.app_state)
                            .await
                    }
                    "serverproperties" => {
                        if let Err(e) = crate::bot::settingsview::run(
                            &self.twilight_client,
                            command,
                            &self.app_state,
                        )
                        .await
                        {
                            eprintln!("Error running settingsview: {}", e);
                        }
                        Ok(())
                    }
                    _ => command
                        .create_response(ctx.http, CreateInteractionResponse::Acknowledge)
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
                };
            }
            Interaction::Component(component) => {
                let (action, id) = component
                    .data
                    .custom_id
                    .split_once(':')
                    .unwrap_or(("unknown", "unknown"));
                if let Some(agent) = self.app_state.find_connection(id).await {
                    match action {
                        "allowflightbutton" => {
                            if let Ok(props) = agent
                                .edit_props(protocol::properties::property::allow_flight)
                                .await
                            {
                                if let Err(e) = crate::bot::settingsview::update_settings_view(
                                    &self.twilight_client,
                                    &component,
                                    &props,
                                    id,
                                )
                                .await
                                {
                                    println!("Error updating settings view: {}", e);
                                } else {
                                    println!("Updated settings view")
                                }
                            } else {
                                println!("Props was not received");
                            }
                            let _response = component
                                .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
                                .await;
                        }
                        _ => {
                            let _response = component
                                .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
                                .await;
                        }
                    }
                } else {
                    eprintln!("No agent found for ID: {}", id);
                }
            }
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
