use crate::appstate::AppState;
use crate::mods::bot::si2tr::si2tr;
use anyhow::{Result, anyhow};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use protocol::query_options::QuerySend;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;
use std::collections::HashSet;
use twilight_model::channel::message::Component;
use twilight_model::channel::message::MessageFlags;
use twilight_model::channel::message::component::ButtonStyle;
use twilight_model::channel::message::component::{
    Checkbox, CheckboxGroup, CheckboxGroupOption, SelectMenu, SelectMenuOption, SelectMenuType,
};
use twilight_model::channel::message::component::{TextDisplay, UnfurledMediaItem};
use twilight_model::http::attachment::Attachment;
use twilight_model::http::interaction::InteractionResponseType;
use twilight_model::http::interaction::*;
use twilight_model::id::Id;
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::message::{ActionRowBuilder, TextDisplayBuilder};
use twilight_util::builder::message::{
    ButtonBuilder, LabelBuilder, SectionBuilder, SelectMenuBuilder, SelectMenuOptionBuilder,
    ThumbnailBuilder,
};

pub async fn builder_modal(
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

    let response = build_monitor(id);

    si2tr(client, &serenity_interaction, &response).await;

    Ok(())
}

pub fn register() -> CreateCommand {
    let id = CreateCommandOption::new(CommandOptionType::String, "name", "Name of socket");
    CreateCommand::new("thumbnail")
        .description("see the header")
        .add_option(id)
}

pub async fn build_view(
    options: HashSet<String>,
    client: &twilight_http::Client,
    serenity_interaction: &serenity::model::application::ModalInteraction,
    appstate: &AppState,
    id: &str,
) -> Result<()> {
    //let png_bytes = STANDARD.decode(dog_base64_string())?;

    let application_id = Id::new(serenity_interaction.application_id.get());
    let interaction_id = Id::new(serenity_interaction.id.get());

    let data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .build();

    let response = InteractionResponse {
        kind: InteractionResponseType::DeferredChannelMessageWithSource, //interaction response creating message that can be edited later
        data: Some(data),
    };

    let interaction_client = client.interaction(application_id);
    if let Err(e) = interaction_client
        .create_response(interaction_id, &serenity_interaction.token, &response)
        .await
    {
        eprintln!("Failed to send twilight response: {}", e);
    }

    let message = interaction_client
        .response(&serenity_interaction.token)
        .await?
        .model()
        .await?;
    let message_id = message.id;
    let channel_id = message.channel_id;

    let agent = appstate
        .find_connection(id)
        .await
        .ok_or_else(|| anyhow!("Agent Not Found"))?;

    let (description, image, query) = agent
        .start_query(options, message_id.get(), channel_id.get())
        .await?;
    let mut attachment = Attachment::from_bytes("server_icon.png".to_string(), image, 1);
    attachment.description("Server Favicon".to_string());
    let mediaitem = UnfurledMediaItem {
        url: "attachment://server_icon.png".to_string(),
        proxy_url: None,
        height: None,
        width: None,
        content_type: None,
    };
    let displaytext = TextDisplay {
        id: None,
        content: format!("# Message Of The Day:\n{}", description),
    };
    let thumbnail = ThumbnailBuilder::new(mediaitem).build();
    let header = SectionBuilder::new(thumbnail)
        .component(displaytext)
        .build();
    let mut components = vec![Component::Section(header)];

    println!(
        "Version: {}",
        query
            .version()
            .unwrap_or(&"No version found pooooop".to_string())
    );

    if let Some(version) = query.version() {
        println!("found version");
        components.push(Component::TextDisplay(
            TextDisplayBuilder::new(format!("# Version:\n{}", version)).build(),
        ));
    }

    if let Some(player_count) = query.player_count() {
        println!("found player count");
        components.push(Component::TextDisplay(
            TextDisplayBuilder::new(format!("# Player Count:\n{}", player_count)).build(),
        ));
    }

    if let Some(player_list) = query.player_list() {
        println!("found player list");
        components.push(Component::TextDisplay(
            TextDisplayBuilder::new(format!("# Player List:\n{}", player_list.join("\n"))).build(),
        ));
    }

    if let Some(map) = query.map() {
        println!("found map");
        components.push(Component::TextDisplay(
            TextDisplayBuilder::new(format!("# Map:\n{}", map)).build(),
        ));
    }

    println!(
        "gamemode: {}",
        query.gamemode().unwrap_or(&"No gamemode found".to_string())
    );
    if let Some(gamemode) = query.gamemode() {
        println!("found gamemode");
        components.push(Component::TextDisplay(
            TextDisplayBuilder::new(format!("# Gamemode:\n{}", gamemode)).build(),
        ));
    }

    if let Some(software) = query.software() {
        println!("found software");
        components.push(Component::TextDisplay(
            TextDisplayBuilder::new(format!("# Software:\n{}", software)).build(),
        ));
    }

    if let Some(plugins) = query.plugins() {
        println!("found plugins");
        components.push(Component::TextDisplay(
            TextDisplayBuilder::new(format!("# Plugins:\n{}", plugins.join("\n"))).build(),
        ))
    }

    if let Some(mods) = query.mods() {
        println!("found mods");
        components.push(Component::TextDisplay(
            TextDisplayBuilder::new(format!("# Mods:\n{}", mods.join("\n"))).build(),
        ))
    }

    interaction_client
        .update_response(&serenity_interaction.token)
        .components(Some(&components))
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .attachments(&vec![attachment])
        .await?;
    Ok(())
}

pub fn build_monitor(id: &str) -> InteractionResponse {
    let version = CheckboxGroupOption {
        value: "version".to_string(),
        label: "Version".to_string(),
        description: Some("What minecraft version".to_string()),
        default: None,
    };
    let player_count = CheckboxGroupOption {
        value: "player count".to_string(),
        label: "Player Count".to_string(),
        description: Some("How many players are on the server".to_string()),
        default: None,
    };
    let player_list = CheckboxGroupOption {
        value: "player list".to_string(),
        label: "Player List".to_string(),
        description: Some("List of players on the server".to_string()),
        default: None,
    };
    let map = CheckboxGroupOption {
        value: "map".to_string(),
        label: "Map".to_string(),
        description: Some("What map is loaded on the server".to_string()),
        default: None,
    };
    let gamemode = CheckboxGroupOption {
        value: "gamemode".to_string(),
        label: "Gamemode".to_string(),
        description: Some("Current Server Gamemode".to_string()),
        default: None,
    };
    let software = CheckboxGroupOption {
        value: "software".to_string(),
        label: "Software".to_string(),
        description: Some("What software is loaded. E.g. Vanilla, Spigot, Paper".to_string()),
        default: None,
    };
    let plugins = CheckboxGroupOption {
        value: "plugins".to_string(),
        label: "Plugins".to_string(),
        description: Some("What plugins are loaded on the server".to_string()),
        default: None,
    };
    let mods = CheckboxGroupOption {
        value: "mods".to_string(),
        label: "Mods".to_string(),
        description: Some("What mods are loaded on the server".to_string()),
        default: None,
    };
    let checkboxgroup = CheckboxGroup {
        id: None,
        custom_id: "checkboxgroupid".to_string(),
        options: vec![
            version,
            player_count,
            player_list,
            map,
            gamemode,
            software,
            plugins,
            mods,
        ],
        min_values: None,
        max_values: None,
        required: Some(true),
    };
    let text = TextDisplayBuilder::new("What would you like to be in your query monitor?").build();
    let label = LabelBuilder::new("Options: ", Component::CheckboxGroup(checkboxgroup)).build();
    let data = InteractionResponseDataBuilder::new()
        .components(vec![Component::TextDisplay(text), Component::Label(label)])
        .content("jello")
        .title("Build Monitor")
        .custom_id(format!("build_query:checkbox_test:{}", id))
        .build();
    InteractionResponse {
        kind: InteractionResponseType::Modal,
        data: Some(data),
    }
}
