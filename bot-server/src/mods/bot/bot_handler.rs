use crate::bot::*;
use crate::mods::bot::props_modals::props_modal;
use crate::mods::*;
use anyhow::{Result, anyhow};
use axum::handler;
use protocol::properties::property;
use serenity::all::{ActionRowComponent, CreateModal};
use serenity::async_trait;
use serenity::builder::{
    CreateActionRow, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::application::{
    ActionRow, Command, InputTextStyle, Interaction, ResolvedOption,
};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use settingscreen::SettingScreen;
use std::env;
use std::str::FromStr;
use twilight_model::channel::Message;
use twilight_model::channel::message::Component;
use twilight_model::channel::message::component::ActionRow as TwilightRow;

pub struct Handler {
    pub app_state: crate::appstate::AppState,
    pub twilight_client: twilight_http::Client,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Err(e) = self.handle_interaction(ctx, interaction).await {
            println!("Error with interaction: {}", e);
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

fn parse_custom_id(id: &str) -> Option<(ComponentAction, &str)> {
    println!("parsing, received: {}", id);
    let mut parts = id.split(':');

    let kind = parts.next()?;
    let value = parts.next()?;
    let server_id = parts.next()?;

    let action = match kind {
        "edit" => {
            let prop = match value {
                "allow_flightbutton" => property::allow_flight,
                "difficultybutton" => property::difficulty,
                "gamemodebutton" => property::gamemode,
                "hardcorebutton" => property::hardcore,
                "whitelistbutton" => property::whitelist,
                "pvpbutton" => property::pvp,
                "generate_structuresbutton" => property::generate_structures,
                "allownetherbutton" => property::allow_nether,
                "spawn-npcbutton" => property::spawn_npcs,
                "spawn-animalsbutton" => property::spawn_animals,
                "spawn-monstersbutton" => property::spawn_monsters,
                _ => {
                    println!("parsing returned none");
                    return None;
                }
            };
            println!("Selected prop");
            ComponentAction::Edit(prop)
        }
        "modal" => {
            let modal = match value {
                "motdbutton" => {
                    props_modal("Message Of The Day", server_id, InputTextStyle::Paragraph)
                }
                "max-playersbutton" => props_modal("Max Players", server_id, InputTextStyle::Short),
                "max-worldbutton" => {
                    props_modal("Max World Size", server_id, InputTextStyle::Short)
                }
                "view-distancebutton" => {
                    props_modal("View Distance", server_id, InputTextStyle::Short)
                }
                "simulation-distancebutton" => {
                    props_modal("Simulation Distance", server_id, InputTextStyle::Short)
                }
                "spawn-protectionbutton" => {
                    props_modal("Spawn Protection", server_id, InputTextStyle::Short)
                }
                _ => return None,
            };
            println!("Selected modal");
            ComponentAction::OpenModal(modal)
        }
        "screen" => {
            let result = SettingScreen::from_str(value);
            match result {
                Ok(screen) => ComponentAction::ChangeScreen(screen),
                Err(e) => {
                    println!("Failed get Setting Screen value from string:");
                    return None;
                }
            }
        }
        _ => return None,
    };

    Some((action, server_id))
}

enum ComponentAction {
    Edit(property),
    OpenModal(CreateModal),
    ChangeScreen(SettingScreen),
}

impl Handler {
    pub async fn handle_interaction(&self, ctx: Context, interaction: Interaction) -> Result<()> {
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
                        crate::bot::settingsview::run(
                            &self.twilight_client,
                            command,
                            &self.app_state,
                        )
                        .await?;
                        Ok(())
                    }
                    _ => command
                        .create_response(ctx.http, CreateInteractionResponse::Acknowledge)
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
                };
            }
            Interaction::Component(component) => {
                let (action, id) = parse_custom_id(&component.data.custom_id)
                    .ok_or_else(|| anyhow!("No custom id found"))?;
                let agent = self
                    .app_state
                    .find_connection(id)
                    .await
                    .ok_or_else(|| anyhow!("Agent not found"))?;
                match action {
                    ComponentAction::Edit(property) => {
                        let props = agent.edit_props(property).await?;
                        crate::bot::settingsview::update_settings_view(
                            &self.twilight_client,
                            component.channel_id.get(),
                            component.message.id.get(),
                            &props,
                            id,
                            None,
                            &component.message,
                        )
                        .await?;
                        let _response = component
                            .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
                            .await;
                    }
                    ComponentAction::OpenModal(modal) => {
                        let _result = component
                            .create_response(&ctx.http, CreateInteractionResponse::Modal(modal))
                            .await;
                    }
                    ComponentAction::ChangeScreen(screen) => {
                        let props = agent.request_props().await?;
                        crate::bot::settingsview::update_settings_view(
                            &self.twilight_client,
                            component.channel_id.get(),
                            component.message.id.get(),
                            &props,
                            id,
                            Some(&screen),
                            &component.message,
                        )
                        .await?;
                        println!("Updated settings view");
                        if component
                            .create_response(ctx.http, CreateInteractionResponse::Acknowledge)
                            .await
                            .is_ok()
                        {
                            println!("acknowledged");
                        }
                    }
                }
            }
            Interaction::Modal(modal) => {
                let (title, id) = modal
                    .data
                    .custom_id
                    .split_once(":")
                    .ok_or_else(|| anyhow!("custom id not split"))?;
                println!("title: {}, id: {}", title, id);
                let ActionRowComponent::InputText(data) = &modal.data.components[0].components[0]
                else {
                    return Ok(());
                };
                let input = &data
                    .value
                    .as_ref()
                    .ok_or_else(|| anyhow!("No input text received"))?;
                let prop = match title {
                    "Message Of The Day" => property::motd(input.to_string()),
                    "Max Players" => {
                        if let Ok(value) = input.parse::<u32>() {
                            print!("value: {}", value);
                            property::max_players(value)
                        } else {
                            println!("Invalid number input");
                            let _result = modal
                                .create_response(
                                    ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Please input a valid number")
                                            .ephemeral(true),
                                    ),
                                )
                                .await;
                            return Ok(());
                        }
                    }
                    "Max World Size" => {
                        if let Ok(value) = input.parse::<u32>() {
                            print!("value: {}", value);
                            property::max_world_size(value)
                        } else {
                            println!("Invalid number input");
                            let _result = modal
                                .create_response(
                                    ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Please input a valid number")
                                            .ephemeral(true),
                                    ),
                                )
                                .await;
                            return Ok(());
                        }
                    }
                    "View Distance" => {
                        if let Ok(value) = input.parse::<u32>() {
                            print!("value: {}", value);
                            property::view_distance(value)
                        } else {
                            println!("Invalid number input");
                            let _result = modal
                                .create_response(
                                    ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Please input a valid number")
                                            .ephemeral(true),
                                    ),
                                )
                                .await;
                            return Ok(());
                        }
                    }
                    "Simulation Distance" => {
                        if let Ok(value) = input.parse::<u32>() {
                            print!("value: {}", value);
                            property::simulation_distance(value)
                        } else {
                            println!("Invalid number input");
                            let _result = modal
                                .create_response(
                                    ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Please input a valid number")
                                            .ephemeral(true),
                                    ),
                                )
                                .await;
                            return Ok(());
                        }
                    }
                    "Spawn Protection" => {
                        if let Ok(value) = input.parse::<u32>() {
                            print!("value: {}", value);
                            property::spawn_protection(value)
                        } else {
                            println!("Invalid number input");
                            let _result = modal
                                .create_response(
                                    ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("Please input a valid number")
                                            .ephemeral(true),
                                    ),
                                )
                                .await;
                            return Ok(());
                        }
                    }
                    _ => {
                        println!("{} Modal Not found", title);
                        return Ok(());
                    }
                };
                let message = modal
                    .message
                    .as_ref()
                    .ok_or_else(|| anyhow!("Message not attatched to any modal"))?;
                let agent = self
                    .app_state
                    .find_connection(id)
                    .await
                    .ok_or_else(|| anyhow!("Agent not found for id: {}", id))?;
                let props = agent.edit_props(prop).await?;
                crate::bot::settingsview::update_settings_view(
                    &self.twilight_client,
                    modal.channel_id.get(),
                    message.id.get(),
                    &props,
                    id,
                    None,
                    &message,
                )
                .await?;

                let _result = modal
                    .create_response(ctx.http, CreateInteractionResponse::Acknowledge)
                    .await;
                println!("Updated settings view");
            }
            _ => println!("uh oh..."),
        }
        Ok(())
    }
}
