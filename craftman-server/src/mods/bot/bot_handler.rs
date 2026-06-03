use crate::bot::*;
use crate::mods::bot::chat_commands::start_chat::start_chat;
use crate::mods::bot::chat_commands::stop_chat::stop_chat;
use crate::mods::bot::server_commands::properties::props_modals::props_modal;
use crate::mods::bot::server_commands::{startserver, stopserver};
use anyhow::{Result, anyhow, bail};
use properties::settingscreen::SettingScreen;
use protocol::properties::Property;
use serenity::all::{ActionRowComponent, CreateModal};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{InputTextStyle, Interaction};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use server_commands::properties;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};
use twilight_model::application::interaction::Interaction as TwilightInteraction;
use twilight_model::application::interaction::InteractionData;
use twilight_model::application::interaction::modal::ModalInteractionComponent;
use uuid::Uuid;

pub struct Handler {
    pub app_state: crate::appstate::AppState,
    pub twilight_client: Arc<twilight_http::Client>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Err(e) = self.handle_interaction(ctx, interaction).await {
            error!("Error with interaction: {}", e);
        }
    }

    #[instrument(skip(self, ctx, ready))]
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let create_commands = vec![
            server_commands::server_command::register_server_command(),
            query_monitor::register(),
            chat_commands::chat_commands_register::register_chat_command(),
            connect_to_server::register(),
        ];

        let commands = if cfg!(debug_assertions) {
            let guild_id = GuildId::new(
                env::var("GUILD_ID")
                    .expect("Expected GUILD_ID in environment")
                    .parse()
                    .expect("GUILD_ID must be an integer"),
            );
            guild_id
                .set_commands(&ctx.http, create_commands)
                .await
                .expect("Could not register commands :(")
        } else {
            serenity::model::application::Command::set_global_commands(&ctx.http, create_commands)
                .await
                .expect("Could not register commands :(")
        };
        info!("Registered the following guild slash commands: {commands:#?}");
    }
}

#[instrument]
fn parse_custom_id(id: &str) -> Option<(ComponentAction, &str)> {
    debug!("parsing");
    let mut parts = id.split(':');

    let kind = parts.next()?;
    let value = parts.next()?;
    let server_id = parts.next()?;
    debug!("Kind: {}", kind);

    let action = match kind {
        "edit" => {
            let prop = match value {
                "allowflightbutton" => Property::AllowFlight,
                "difficultybutton" => Property::Difficulty,
                "gamemodebutton" => Property::Gamemode,
                "hardcorebutton" => Property::Hardcore,
                "whitelistbutton" => Property::Whitelist,
                "pvpbutton" => Property::PVP,
                "generatestructuresbutton" => Property::GenerateStructures,
                "allownetherbutton" => Property::AllowNether,
                "spawn-npcbutton" => Property::SpawnNPC,
                "spawn-animalsbutton" => Property::SpawnAnimals,
                "spawn-monstersbutton" => Property::SpawnMonsters,
                _ => {
                    error!("parsing returned none");
                    return None;
                }
            };
            debug!("Selected prop");
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
            debug!("Selected modal");
            ComponentAction::OpenModal(modal)
        }
        "screen" => {
            let result = SettingScreen::from_str(value);
            match result {
                Ok(screen) => ComponentAction::ChangeScreen(screen),
                Err(_e) => {
                    error!("Failed get Setting Screen value from string:");
                    return None;
                }
            }
        }
        _ => return None,
    };

    Some((action, server_id))
}

enum ComponentAction {
    Edit(Property),
    OpenModal(CreateModal),
    ChangeScreen(SettingScreen),
}

impl Handler {
    pub async fn handle_interaction(&self, ctx: Context, interaction: Interaction) -> Result<()> {
        match interaction {
            Interaction::Command(command) => {
                tracing::debug!("Received command interaction: {command:#?}");
                let command_name = command.data.name.as_str();
                match command_name {
                    "server" => match command.data.options[0].name.as_str() {
                        "start" => {
                            startserver::start_mc_server(&ctx, &command, &self.app_state).await?;
                        }
                        "stop" => {
                            stopserver::stop_minecraft_server(&ctx, &command, &self.app_state)
                                .await?;
                        }
                        "properties" => {
                            server_commands::properties::settingsview::run(
                                &ctx,
                                &self.twilight_client,
                                command,
                                &self.app_state,
                            )
                            .await?;
                        }
                        _ => {}
                    },
                    "chat" => match command.data.options[0].name.as_str() {
                        "start" => {
                            start_chat(
                                &ctx,
                                &command,
                                &self.app_state,
                                self.twilight_client.clone(),
                            )
                            .await?;
                        }
                        "stop" => {
                            stop_chat(&ctx, &command, &self.app_state).await?;
                        }
                        "set" => {
                            chat_commands::chat_channel::set_chat_channel(
                                &ctx,
                                &command,
                                &self.app_state,
                            )
                            .await?;
                        }
                        "say" | "command" => {
                            chat_commands::message_chat::send_to_minecraft(
                                &ctx,
                                &command,
                                &self.app_state,
                                command.data.options[0].name.as_str(),
                            )
                            .await?;
                        }
                        _ => {}
                    },
                    "monitor" => {
                        crate::bot::query_monitor::builder_modal(
                            &ctx,
                            &self.twilight_client,
                            command,
                            &self.app_state,
                        )
                        .await?;
                    }
                    "verify" => {
                        connect_to_server::connect_server(&ctx, &command, &self.app_state).await?;
                    }
                    _ => {
                        command
                            .create_response(
                                ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new().content(format!(
                                        "Could not find command: {}",
                                        command_name
                                    )),
                                ),
                            )
                            .await?;
                    }
                };
            }
            Interaction::Component(component) => {
                let (action, id_str) = parse_custom_id(&component.data.custom_id)
                    .ok_or_else(|| anyhow!("No custom id found"))?;
                let id = Uuid::from_str(id_str)?;
                let agent = self.app_state.find_connection(&id)?;
                match action {
                    ComponentAction::Edit(property) => {
                        let props = agent.edit_props(property).await?;
                        server_commands::properties::settingsview::update_settings_view(
                            &self.twilight_client,
                            component.channel_id.get(),
                            component.message.id.get(),
                            &props,
                            id,
                            None,
                            &component.message,
                        )
                        .await?;
                        component
                            .create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
                            .await?;
                    }
                    ComponentAction::OpenModal(modal) => {
                        component
                            .create_response(&ctx.http, CreateInteractionResponse::Modal(modal))
                            .await?;
                    }
                    ComponentAction::ChangeScreen(screen) => {
                        let props = agent.request_props().await?;
                        server_commands::properties::settingsview::update_settings_view(
                            &self.twilight_client,
                            component.channel_id.get(),
                            component.message.id.get(),
                            &props,
                            id,
                            Some(&screen),
                            &component.message,
                        )
                        .await?;
                        tracing::debug!("Updated settings view");
                        component
                            .create_response(ctx.http, CreateInteractionResponse::Acknowledge)
                            .await?;
                    }
                }
            }
            Interaction::Modal(modal, raw_json) => {
                let (action, title, id_str) = parse_modal_custom_id(&modal.data.custom_id)
                    .ok_or_else(|| anyhow!("Custom_id not good"))?;
                let id = Uuid::from_str(id_str)?;
                tracing::debug!("Title: {}  id: {}", title, id);
                match action {
                    ModalAction::EditProp => {
                        tracing::debug!("title: {}, id: {}", title, id);
                        let ActionRowComponent::InputText(data) =
                            &modal.data.components[0].components[0]
                        else {
                            return Ok(());
                        };
                        let input = &data
                            .value
                            .as_ref()
                            .ok_or_else(|| anyhow!("No input text received"))?;
                        let prop = match title {
                            "Message Of The Day" => Property::MOTD(input.to_string()),
                            "Max Players" => {
                                if let Ok(value) = input.parse::<u32>() {
                                    Property::MaxPlayers(value)
                                } else {
                                    tracing::debug!("Invalid number input");
                                    modal
                                        .create_response(
                                            ctx.http,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .content("Please input a valid number")
                                                    .ephemeral(true),
                                            ),
                                        )
                                        .await?;
                                    return Ok(());
                                }
                            }
                            "Max World Size" => {
                                if let Ok(value) = input.parse::<u32>() {
                                    Property::MaxWorldSize(value)
                                } else {
                                    modal
                                        .create_response(
                                            ctx.http,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .content("Please input a valid number")
                                                    .ephemeral(true),
                                            ),
                                        )
                                        .await?;
                                    return Ok(());
                                }
                            }
                            "View Distance" => {
                                if let Ok(value) = input.parse::<u32>() {
                                    Property::ViewDistance(value)
                                } else {
                                    modal
                                        .create_response(
                                            ctx.http,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .content("Please input a valid number")
                                                    .ephemeral(true),
                                            ),
                                        )
                                        .await?;
                                    return Ok(());
                                }
                            }
                            "Simulation Distance" => {
                                if let Ok(value) = input.parse::<u32>() {
                                    Property::SimulationDistance(value)
                                } else {
                                    modal
                                        .create_response(
                                            ctx.http,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .content("Please input a valid number")
                                                    .ephemeral(true),
                                            ),
                                        )
                                        .await?;
                                    return Ok(());
                                }
                            }
                            "Spawn Protection" => {
                                if let Ok(value) = input.parse::<u32>() {
                                    Property::SpawnProtection(value)
                                } else {
                                    modal
                                        .create_response(
                                            ctx.http,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .content("Please input a valid number")
                                                    .ephemeral(true),
                                            ),
                                        )
                                        .await?;
                                    return Ok(());
                                }
                            }
                            _ => {
                                return Ok(());
                            }
                        };
                        let message = modal
                            .message
                            .as_ref()
                            .ok_or_else(|| anyhow!("Message not attatched to any modal"))?;
                        let agent = self.app_state.find_connection(&id)?;
                        let props = agent.edit_props(prop).await?;
                        properties::settingsview::update_settings_view(
                            &self.twilight_client,
                            modal.channel_id.get(),
                            message.id.get(),
                            &props,
                            id,
                            None,
                            message,
                        )
                        .await?;

                        modal
                            .create_response(ctx.http, CreateInteractionResponse::Acknowledge)
                            .await?;
                        tracing::debug!("Updated settings view");
                    }
                    ModalAction::BuildQuery => {
                        let twilight_interaction: TwilightInteraction =
                            serde_json::from_value(raw_json)?;
                        let InteractionData::ModalSubmit(interaction_data) = twilight_interaction
                            .data
                            .ok_or_else(|| anyhow!("Conversion to inderaction data failed"))?
                        else {
                            bail!("Not modalsubmit I guess");
                        };
                        let ModalInteractionComponent::Label(label) =
                            &interaction_data.components[1]
                        else {
                            bail!("Not label I guess");
                        };
                        let ModalInteractionComponent::CheckboxGroup(checkbox_group) =
                            &*label.component
                        else {
                            bail!("Not a checkbox group I gess");
                        };
                        query_monitor::build_view(
                            checkbox_group.values.clone().into_iter().collect(),
                            &self.twilight_client,
                            &modal,
                            &self.app_state,
                            id,
                        )
                        .await?;
                    }
                }
            }
            _ => tracing::error!("Unhandled action..."),
        }
        Ok(())
    }
}

enum ModalAction {
    EditProp,
    BuildQuery,
}

#[instrument]
fn parse_modal_custom_id(id: &str) -> Option<(ModalAction, &str, &str)> {
    debug!("parsing");
    let mut parts = id.split(':');

    let kind = parts.next()?;
    let value = parts.next()?;
    let server_id = parts.next()?;

    let action = match kind {
        "edit_props" => {
            debug!("Selected prop");
            ModalAction::EditProp
        }
        "build_query" => {
            debug!("Build query");
            ModalAction::BuildQuery
        }
        _ => {
            error!("No action found for this parse");
            return None;
        }
    };
    Some((action, value, server_id))
}
