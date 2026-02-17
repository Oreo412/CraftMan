use crate::appstate::AppState;
use crate::mods::bot::settingscreen::SettingScreen;
use anyhow::Result;
use anyhow::anyhow;
use serenity::all::ActionRowComponent;
use serenity::builder::*;
use serenity::model::application::CommandOptionType;
use std::collections::HashMap;
use std::error::Error;
use twilight_http::Client;
use twilight_http::client::InteractionClient;
use twilight_model::channel::message::MessageFlags;
use twilight_model::channel::message::component::*;
use twilight_model::channel::message::component::*;
use twilight_model::http::interaction::*;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, MessageMarker};
use twilight_util::builder::message::*;

pub async fn run(
    client: &twilight_http::Client,
    serenity_interaction: serenity::model::application::CommandInteraction,
    appstate: &AppState,
) -> Result<()> {
    println!("received");
    let id = serenity_interaction
        .data
        .options
        .iter()
        .find(|option| option.name == "name")
        .ok_or_else(|| anyhow!("No Id"))?
        .value
        .as_str()
        .ok_or_else(|| anyhow!("No Id"))?;
    let agent = appstate
        .find_connection(id)
        .await
        .ok_or_else(|| anyhow!("Agent Not Found"))?;

    let props = agent.request_props().await?;
    let mut responsetest = InteractionResponseData::default();
    responsetest.components = Some(build_settings_view(&props, id, &SettingScreen::Gameplay)?);
    println!("Added components to response");
    responsetest.flags = Some(MessageFlags::IS_COMPONENTS_V2);
    let response = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(responsetest),
    };
    println!("Constructed interaction response");

    si2tr(client, &serenity_interaction, &response).await;
    println!("got to here...");

    Ok(())
}

fn screen_from_message(message: &serenity::model::channel::Message) -> Option<&SettingScreen> {
    let last_row = message.components.last()?;

    let components = &last_row.components;

    for component in components {
        if let ActionRowComponent::Button(button) = component {
            if button.disabled {
                return match button.label.as_deref()? {
                    "World Generation" => Some(&SettingScreen::WorldGeneration),
                    "Gameplay" => Some(&SettingScreen::Gameplay),
                    "Admin" => Some(&SettingScreen::Admin),
                    _ => None,
                };
            }
        }
    }

    None
}

pub fn register() -> CreateCommand {
    let id = CreateCommandOption::new(CommandOptionType::String, "name", "Name of socket");
    CreateCommand::new("serverproperties")
        .description("edit server properties")
        .add_option(id)
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
        .custom_id(format!("allowflightbutton:{}", id))
        .label(label)
        .build();

    SectionBuilder::new(button).component(displaytext).build()
}

pub fn build_settings_view(
    props: &HashMap<String, String>,
    id: &str,
    screen: &SettingScreen,
) -> Result<Vec<Component>> {
    let mut properties = match screen {
        SettingScreen::WorldGeneration => {
            let generate_value = props
                .get("generate-structures")
                .ok_or_else(|| anyhow!("generate-structures not found"))?
                .parse::<bool>()?;
            let allow_nether_value = props
                .get("allow-nether")
                .ok_or_else(|| anyhow!("allow nether not found"))?
                .parse::<bool>()?;
            let max_world_value = props
                .get("max-world-size")
                .ok_or_else(|| anyhow!("max world size not found"))?
                .parse::<u32>()?;
            let spawn_npc_value = props
                .get("spawn-npcs")
                .ok_or_else(|| anyhow!("spawn npcs not found"))?
                .parse::<bool>()?;
            let spawn_animals_value = props
                .get("spawn-animals")
                .ok_or_else(|| anyhow!("spawn animals not found"))?
                .parse::<bool>()?;
            let spawn_monsters_value = props
                .get("spawn-monsters")
                .ok_or_else(|| anyhow!("spawn monsters not found"))?
                .parse::<bool>()?;
            vec![
                Component::Section(generate_structures(generate_value, id)),
                Component::Section(max_world_size(max_world_value, id)),
                Component::Section(allow_nether(allow_nether_value, id)),
                Component::Section(spawn_npcs(spawn_npc_value, id)),
                Component::Section(spawn_animals(spawn_animals_value, id)),
                Component::Section(spawn_monsters(spawn_monsters_value, id)),
            ]
        }
        SettingScreen::Gameplay => {
            let allow_flight_value = props
                .get("allow-flight")
                .ok_or_else(|| anyhow!("allow-flight not found"))?
                .parse::<bool>()?;
            let difficulty_value = props
                .get("difficulty")
                .ok_or_else(|| anyhow!("difficulty not found"))?;
            let gamemode_value = props
                .get("gamemode")
                .ok_or_else(|| anyhow!("Gamemode not found"))?;
            let hardcore_value = props
                .get("hardcore")
                .ok_or_else(|| anyhow!("hardcore not found"))?
                .parse::<bool>()?;
            let spawn_protection_value = props
                .get("spawn-protection")
                .ok_or_else(|| anyhow!("spawn protection not found"))?
                .parse::<u32>()?;
            let pvp_value = props
                .get("pvp")
                .ok_or_else(|| anyhow!("pvp not found"))?
                .parse::<bool>()?;
            vec![
                Component::Section(allow_flight(allow_flight_value, id)),
                Component::Section(difficulty(difficulty_value, id)),
                Component::Section(gamemode(gamemode_value, id)),
                Component::Section(hardcore(hardcore_value, id)),
                Component::Section(spawn_protection(spawn_protection_value, id)),
                Component::Section(pvp(pvp_value, id)),
            ]
        }
        SettingScreen::Admin => {
            let whitelist_value = props
                .get("white-list")
                .ok_or_else(|| anyhow!("whitelist not found"))?
                .parse::<bool>()?;

            let motd_value = props.get("motd").ok_or_else(|| anyhow!("motd not found"))?;
            let max_players_value = props
                .get("max-players")
                .ok_or_else(|| anyhow!("max players not found"))?
                .parse::<u32>()?;

            let view_distance_value = props
                .get("view-distance")
                .ok_or_else(|| anyhow!("view distance not found"))?
                .parse::<u32>()?;
            let simulation_distance_value = props
                .get("simulation-distance")
                .ok_or_else(|| anyhow!("simulation distance not found"))?
                .parse::<u32>()?;
            vec![
                Component::Section(whitelist(whitelist_value, id)),
                Component::Section(motd(motd_value, id)),
                Component::Section(max_players(max_players_value, id)),
                Component::Section(view_distance(view_distance_value, id)),
                Component::Section(simulation_distance(simulation_distance_value, id)),
            ]
        }
    };
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
    properties.push(Component::ActionRow(last_row));
    Ok(properties)
}

pub async fn update_settings_view(
    client: &twilight_http::Client,
    channel_id: u64,
    message_id: u64,
    props: &HashMap<String, String>,
    id: &str,
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

pub async fn si2tr<'a>(
    client: &'a twilight_http::Client,
    interaction: &'a serenity::model::application::CommandInteraction,
    response: &'a InteractionResponse,
) {
    println!("Sending response via twilight");
    let application_id = Id::new(interaction.application_id.get());
    let interaction_id = Id::new(interaction.id.get());
    if let Err(e) = client
        .interaction(application_id)
        .create_response(interaction_id, &interaction.token, response)
        .into_future()
        .await
    {
        eprintln!("Failed to send twilight response: {}", e);
    }
}

pub fn difficulty(difficulty: &String, id: &str) -> Section {
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

pub fn gamemode(gamemode: &String, id: &str) -> Section {
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
        .custom_id(format!("edit:generate-structuresbutton:{}", id))
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
