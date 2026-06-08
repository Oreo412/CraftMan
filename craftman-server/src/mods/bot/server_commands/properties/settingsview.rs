use crate::appstate::AppState;
use crate::mods::bot::get_guild::get_guild;
use crate::mods::bot::server_commands::properties::settingscreen::SettingScreen;
use crate::mods::bot::si2tr::si2tr;
use anyhow::Result;
use anyhow::anyhow;
use serenity::all::ActionRowComponent;
use serenity::all::Context;
use serenity::all::CreateInteractionResponse;
use serenity::all::CreateInteractionResponseMessage;
use std::collections::HashMap;
use twilight_model::channel::message::MessageFlags;
use twilight_model::channel::message::component::*;
use twilight_model::id::Id;
use twilight_util::builder::interaction_response::ChannelMessageBuilder;
use twilight_util::builder::message::*;
use uuid::Uuid;

pub async fn run(
    ctx: &Context,
    client: &twilight_http::Client,
    serenity_interaction: serenity::model::application::CommandInteraction,
    appstate: &AppState,
) -> Result<()> {
    let id = appstate.find_id_by_guild(get_guild(ctx, &serenity_interaction).await?)?;
    let agent = if let Ok(agent) =
        appstate.find_connection_by_guild(get_guild(ctx, &serenity_interaction).await?)
    {
        agent
    } else {
        let response = CreateInteractionResponseMessage::new();
        serenity_interaction.create_response(&ctx.http, CreateInteractionResponse::Message(response.content("Unable to find agent. Please either start agent or verify a new agent with /verify"))).await?;
        return Ok(());
    };
    let props = agent.request_props().await?;
    let response = ChannelMessageBuilder::new()
        .components(build_settings_view(&props, id, &SettingScreen::Gameplay)?)
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .build();

    si2tr(client, &serenity_interaction, &response).await;

    Ok(())
}

fn screen_from_message(message: &serenity::model::channel::Message) -> Option<&SettingScreen> {
    let last_row = message.components.last()?;

    let components = &last_row.components;

    for component in components {
        if let ActionRowComponent::Button(button) = component
            && button.disabled
        {
            return match button.label.as_deref()? {
                "World Generation" => Some(&SettingScreen::WorldGeneration),
                "Gameplay" => Some(&SettingScreen::Gameplay),
                "Admin" => Some(&SettingScreen::Admin),
                _ => None,
            };
        }
    }

    None
}

pub fn allow_flight(allow: bool, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "Allow Flight: ".to_string(),
    };

    let (style, label) = if allow {
        (ButtonStyle::Success, "On")
    } else {
        (ButtonStyle::Danger, "Off")
    };

    let button = ButtonBuilder::new(style)
        .custom_id(format!("edit:allowflightbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn build_settings_view(
    props: &HashMap<String, String>,
    uuid: Uuid,
    screen: &SettingScreen,
) -> Result<Vec<Component>> {
    let id = &uuid.to_string();
    let mut properties_message = Vec::<Component>::new();

    match screen {
        SettingScreen::WorldGeneration => {
            if let Some(generate_value) = props.get("generate-structure") {
                properties_message
                    .push(generate_structures(generate_value.parse::<bool>()?, id).into());
            }

            if let Some(max_world_value) = props.get("max-world-size") {
                properties_message.push(max_world_size(max_world_value.parse::<u32>()?, id).into());
            }

            if let Some(allow_nether_value) = props.get("allow-nether") {
                properties_message
                    .push(allow_nether(allow_nether_value.parse::<bool>()?, id).into());
            }

            if let Some(spawn_npc_value) = props.get("spawn-npcs") {
                properties_message.push(spawn_npcs(spawn_npc_value.parse::<bool>()?, id).into());
            }

            if let Some(spawn_animals_value) = props.get("spawn-animals") {
                properties_message
                    .push(spawn_animals(spawn_animals_value.parse::<bool>()?, id).into());
            }

            if let Some(spawn_monsters_value) = props.get("spawn-monsters") {
                properties_message
                    .push(spawn_monsters(spawn_monsters_value.parse::<bool>()?, id).into());
            }
        }

        SettingScreen::Gameplay => {
            if let Some(allow_flight_value) = props.get("allow-flight") {
                properties_message
                    .push(allow_flight(allow_flight_value.parse::<bool>()?, id).into());
            }

            if let Some(difficulty_value) = props.get("difficulty") {
                properties_message.push(difficulty(difficulty_value, id).into());
            }

            if let Some(gamemode_value) = props.get("gamemode") {
                properties_message.push(gamemode(gamemode_value, id).into());
            }

            if let Some(hardcore_value) = props.get("hardcore") {
                properties_message.push(hardcore(hardcore_value.parse::<bool>()?, id).into());
            }

            if let Some(spawn_protection_value) = props.get("spawn-protection") {
                properties_message
                    .push(spawn_protection(spawn_protection_value.parse::<u32>()?, id).into());
            }

            if let Some(pvp_value) = props.get("pvp") {
                properties_message.push(pvp(pvp_value.parse::<bool>()?, id).into());
            }
        }

        SettingScreen::Admin => {
            if let Some(whitelist_value) = props.get("white-list") {
                properties_message.push(whitelist(whitelist_value.parse::<bool>()?, id).into());
            }

            if let Some(motd_value) = props.get("motd") {
                properties_message.push(motd(motd_value, id).into());
            }

            if let Some(max_players_value) = props.get("max-players") {
                properties_message.push(max_players(max_players_value.parse::<u32>()?, id).into());
            }

            if let Some(view_distance_value) = props.get("view-distance") {
                properties_message
                    .push(view_distance(view_distance_value.parse::<u32>()?, id).into());
            }

            if let Some(simulation_distance_value) = props.get("simulation-distance") {
                properties_message.push(
                    simulation_distance(simulation_distance_value.parse::<u32>()?, id).into(),
                );
            }
        }
    }
    let world_button = ButtonBuilder::new(ButtonStyle::Secondary)
        .custom_id(format!("screen:world:{}", id))
        .label("World Generation")
        .disabled(screen == &SettingScreen::WorldGeneration)
        .build();
    let gameplay_button = ButtonBuilder::new(ButtonStyle::Secondary)
        .custom_id(format!("screen:gameplay:{}", id))
        .label("Gameplay")
        .disabled(screen == &SettingScreen::Gameplay)
        .build();
    let admin_button = ButtonBuilder::new(ButtonStyle::Secondary)
        .custom_id(format!("screen:admin:{}", id))
        .label("Admin")
        .disabled(screen == &SettingScreen::Admin)
        .build();
    let last_row = ActionRowBuilder::new()
        .component(world_button)
        .component(gameplay_button)
        .component(admin_button)
        .build();
    properties_message.push(last_row.into());

    Ok(properties_message)
}

pub async fn update_settings_view(
    client: &twilight_http::Client,
    channel_id: u64,
    message_id: u64,
    props: &HashMap<String, String>,
    id: Uuid,
    screen: Option<&SettingScreen>,
    message: &serenity::model::channel::Message,
) -> Result<()> {
    let new_screen = (screen.or_else(|| screen_from_message(message).or(None)))
        .ok_or_else(|| anyhow!("Tried to get screen from message but failed: "))?;
    let components = build_settings_view(props, id, new_screen);

    client
        .update_message(Id::new(channel_id), Id::new(message_id))
        .components(Some(&components?))
        .await?;
    Ok(())
}

pub fn difficulty(difficulty: &str, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "Difficulty: ".to_string(),
    };

    let button = ButtonBuilder::new(ButtonStyle::Primary)
        .custom_id(format!("edit:difficultybutton:{}", id))
        .label(capitalize(difficulty))
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

pub fn gamemode(gamemode: &str, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "Gamemode: ".to_string(),
    };

    let button = ButtonBuilder::new(ButtonStyle::Primary)
        .custom_id(format!("edit:gamemodebutton:{}", id))
        .label(capitalize(gamemode))
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn hardcore(is_hard: bool, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "Hardcore: ".to_string(),
    };

    let (style, label) = if is_hard {
        (ButtonStyle::Danger, "On")
    } else {
        (ButtonStyle::Primary, "Off")
    };

    let button = ButtonBuilder::new(style)
        .custom_id(format!("edit:hardcorebutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn allow_nether(allow: bool, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "Allow Nether: ".to_string(),
    };

    let (style, label) = if allow {
        (ButtonStyle::Success, "On")
    } else {
        (ButtonStyle::Danger, "Off")
    };

    let button = ButtonBuilder::new(style)
        .custom_id(format!("edit:allownetherbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn whitelist(whitelist: bool, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "White-list: ".to_string(),
    };

    let (style, label) = if whitelist {
        (ButtonStyle::Success, "On")
    } else {
        (ButtonStyle::Danger, "Off")
    };

    let button = ButtonBuilder::new(style)
        .custom_id(format!("edit:whitelistbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn pvp(pvp: bool, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "PVP: ".to_string(),
    };

    let (style, label) = if pvp {
        (ButtonStyle::Danger, "On")
    } else {
        (ButtonStyle::Success, "Off")
    };

    let button = ButtonBuilder::new(style)
        .custom_id(format!("edit:pvpbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn generate_structures(generate: bool, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "Generate Structures: ".to_string(),
    };

    let (style, label) = if generate {
        (ButtonStyle::Success, "On")
    } else {
        (ButtonStyle::Danger, "Off")
    };

    let button = ButtonBuilder::new(style)
        .custom_id(format!("edit:generatestructuresbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn max_players(max: u32, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: format!("Max Players: {}", max),
    };

    let (style, label) = (ButtonStyle::Primary, "Edit");

    let button = ButtonBuilder::new(style)
        .custom_id(format!("modal:max-playersbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn motd(message: &str, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: format!("Message Of The Day: {}", message),
    };

    let (style, label) = (ButtonStyle::Primary, "Edit");

    let button = ButtonBuilder::new(style)
        .custom_id(format!("modal:motdbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn max_world_size(size: u32, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: format!("Max World Size: {}", size),
    };

    let (style, label) = (ButtonStyle::Primary, "Edit");

    let button = ButtonBuilder::new(style)
        .custom_id(format!("modal:max-worldbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn view_distance(distance: u32, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: format!("View Distance: {}", distance),
    };

    let (style, label) = (ButtonStyle::Primary, "Edit");

    let button = ButtonBuilder::new(style)
        .custom_id(format!("modal:view-distancebutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn simulation_distance(distance: u32, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: format!("Simulation Distance: {}", distance),
    };

    let (style, label) = (ButtonStyle::Primary, "Edit");

    let button = ButtonBuilder::new(style)
        .custom_id(format!("modal:simulation-distancebutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn spawn_protection(distance: u32, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: format!("Spawn Protection: {}", distance),
    };

    let (style, label) = (ButtonStyle::Primary, "Edit");

    let button = ButtonBuilder::new(style)
        .custom_id(format!("modal:spawn-protectionbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn spawn_npcs(spawn: bool, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "Spawn NPC's: ".to_string(),
    };

    let (style, label) = if spawn {
        (ButtonStyle::Success, "On")
    } else {
        (ButtonStyle::Danger, "Off")
    };

    let button = ButtonBuilder::new(style)
        .custom_id(format!("edit:spawn-npcbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn spawn_animals(spawn: bool, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "Spawn animals: ".to_string(),
    };

    let (style, label) = if spawn {
        (ButtonStyle::Success, "On")
    } else {
        (ButtonStyle::Danger, "Off")
    };

    let button = ButtonBuilder::new(style)
        .custom_id(format!("edit:spawn-animalsbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn spawn_monsters(spawn: bool, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "Spawn Monsters: ".to_string(),
    };

    let (style, label) = if spawn {
        (ButtonStyle::Success, "On")
    } else {
        (ButtonStyle::Danger, "Off")
    };

    let button = ButtonBuilder::new(style)
        .custom_id(format!("edit:spawn-monstersbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}
