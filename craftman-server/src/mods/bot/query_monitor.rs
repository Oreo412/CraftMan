use crate::appstate::AppState;
use crate::mods::bot::get_guild::get_guild;
use crate::mods::bot::si2tr::si2tr;
use anyhow::Result;
use protocol::query_options::{QueryStatus, ServerStatus};
use serenity::all::Context;
use serenity::all::CreateInteractionResponse;
use serenity::all::CreateInteractionResponseMessage;
use serenity::builder::CreateCommand;
use std::collections::HashSet;
use twilight_model::channel::message::Component;
use twilight_model::channel::message::MessageFlags;
use twilight_model::channel::message::component::{TextDisplay, UnfurledMediaItem};
use twilight_model::http::attachment::Attachment;
use twilight_model::http::interaction::InteractionResponseType;
use twilight_model::http::interaction::*;
use twilight_model::id::{
    Id,
    marker::{ChannelMarker, MessageMarker},
};
use twilight_util::builder::interaction_response::ModalBuilder;
use twilight_util::builder::message::{
    CheckboxGroupBuilder, CheckboxGroupOptionBuilder, TextDisplayBuilder,
};
use twilight_util::builder::message::{LabelBuilder, SectionBuilder, ThumbnailBuilder};
use uuid::Uuid;

static DEFAULT_ICON: &[u8] = include_bytes!("../../../assets/default_icon.png");

pub async fn builder_modal(
    ctx: &Context,
    client: &twilight_http::Client,
    serenity_interaction: serenity::model::application::CommandInteraction,
    appstate: &AppState,
) -> Result<()> {
    if appstate
        .find_connection_by_guild(get_guild(ctx, &serenity_interaction).await?)
        .is_err()
    {
        let response = CreateInteractionResponseMessage::new();
        serenity_interaction.create_response(&ctx.http, CreateInteractionResponse::Message(response.content("Unable to find agent. Please either start agent or verify a new agent with /verify"))).await?;
        return Ok(());
    };

    let id = appstate.find_id_by_guild(get_guild(ctx, &serenity_interaction).await?)?;

    let response = build_monitor(id);

    si2tr(client, &serenity_interaction, &response).await;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("monitor").description("see the header")
}

pub async fn build_view(
    options: HashSet<String>,
    client: &twilight_http::Client,
    serenity_interaction: &serenity::model::application::ModalInteraction,
    appstate: &AppState,
    uuid: Uuid,
) -> Result<()> {
    //let png_bytes = STANDARD.decode(dog_base64_string())?;

    let application_id = Id::new(serenity_interaction.application_id.get());
    let interaction_id = Id::new(serenity_interaction.id.get());

    let response = InteractionResponse {
        kind: InteractionResponseType::DeferredChannelMessageWithSource,
        data: Some(InteractionResponseData {
            flags: Some(MessageFlags::IS_COMPONENTS_V2),
            ..Default::default()
        }),
    };

    let interaction_client = client.interaction(application_id);
    if let Err(e) = interaction_client
        .create_response(interaction_id, &serenity_interaction.token, &response)
        .await
    {
        tracing::warn!("Failed to send twilight response: {}", e);
    }

    let message = interaction_client
        .response(&serenity_interaction.token)
        .await?
        .model()
        .await?;
    let message_id = message.id;
    let channel_id = message.channel_id;

    let agent = appstate.find_connection(&uuid)?;

    let (description, image, status) = agent.new_query(options, message_id, channel_id).await?;
    let filename = format!("{}.png", uuid::Uuid::new_v4());
    let mut attachment =
        Attachment::from_bytes(filename.clone(), image.unwrap_or(DEFAULT_ICON.to_vec()), 1);
    attachment.description("Server Favicon".to_string());
    let mediaitem = UnfurledMediaItem {
        url: format!("attachment://{}", filename),
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
    let statustext = if status == ServerStatus::ServerOffline {
        TextDisplayBuilder::new("🔴 **Offline**").build()
    } else {
        TextDisplayBuilder::new("🟢 **Online**").build()
    };
    let mut components = vec![statustext.into(), header.into()];

    if let ServerStatus::ServerOnline(query) = status {
        components.append(&mut build_monitor_display(query)?);
        interaction_client
            .update_response(&serenity_interaction.token)
            .components(Some(&components))
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .attachments(&[attachment])
            .await?;
    } else {
        interaction_client
            .update_response(&serenity_interaction.token)
            .components(Some(&components))
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .attachments(&[attachment])
            .await?;
    }
    Ok(())
}

pub fn build_monitor(uuid: Uuid) -> InteractionResponse {
    let id = &uuid.to_string();
    let version = CheckboxGroupOptionBuilder::new("version", "Version")
        .description("What Minecraft version is this server")
        .build();

    let player_count = CheckboxGroupOptionBuilder::new("player count", "Player Count")
        .description("How many players are on the server")
        .build();

    let player_list = CheckboxGroupOptionBuilder::new("player list", "Player List")
        .description("List of players on the server")
        .build();

    let map = CheckboxGroupOptionBuilder::new("map", "Map")
        .description("What map is loaded on the server")
        .build();

    let gamemode = CheckboxGroupOptionBuilder::new("gamemode", "Gamemode")
        .description("Current Server Gamemode")
        .build();

    let software = CheckboxGroupOptionBuilder::new("software", "Software")
        .description("What software is loaded. E.g. Vanilla, Spigot, Paper")
        .build();

    let plugins = CheckboxGroupOptionBuilder::new("plugins", "Plugins")
        .description("What plugins are loaded on the server")
        .build();

    let mods = CheckboxGroupOptionBuilder::new("mods", "Mods")
        .description("What mods are loaded on the server")
        .build();

    let checkboxgroup = CheckboxGroupBuilder::new("checkboxgroupid")
        .option(version)
        .option(player_count)
        .option(player_list)
        .option(map)
        .option(gamemode)
        .option(software)
        .option(plugins)
        .option(mods)
        .required(true)
        .build();

    let text = TextDisplayBuilder::new("What would you like to be in your query monitor?").build();
    let label = LabelBuilder::new("Options: ", checkboxgroup.into()).build();
    let components: Vec<Component> = vec![text.into(), label.into()];
    ModalBuilder::new(
        format!("build_query:checkbox_test:{}", id),
        "Build Monitor",
        components,
    )
    .build()
}

pub async fn update_monitor(
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    status: ServerStatus,
    client: &twilight_http::Client,
) -> Result<()> {
    if let ServerStatus::ServerOnline(query) = status {
        let message = client
            .message(channel_id, message_id)
            .await?
            .model()
            .await?;
        let mut components = vec![
            TextDisplayBuilder::new("🟢 **Online**").build().into(),
            message.components[1].clone(),
        ];
        components.append(&mut build_monitor_display(query)?);

        client
            .update_message(channel_id, message_id)
            .components(Some(&components))
            .await?;
    } else {
        let message = client
            .message(channel_id, message_id)
            .await?
            .model()
            .await?;
        let components = vec![
            TextDisplayBuilder::new("🔴 **Offline**").build().into(),
            message.components[1].clone(),
        ];

        client
            .update_message(channel_id, message_id)
            .components(Some(&components))
            .await?;
    }
    Ok(())
}

pub fn build_monitor_display(query: QueryStatus) -> Result<Vec<Component>> {
    let mut components = Vec::new();
    if let Some(version) = query.version() {
        components.push(
            TextDisplayBuilder::new(format!("# Version:\n{}", version))
                .build()
                .into(),
        );
    }

    if let Some(player_count) = query.player_count() {
        components.push(
            TextDisplayBuilder::new(format!("# Player Count:\n{}", player_count))
                .build()
                .into(),
        );
    }

    if let Some(player_list) = query.player_list() {
        components.push(
            TextDisplayBuilder::new(format!("# Player List:\n{}", player_list.join("\n")))
                .build()
                .into(),
        );
    }

    if let Some(map) = query.map() {
        components.push(
            TextDisplayBuilder::new(format!("# Map:\n{}", map))
                .build()
                .into(),
        );
    }

    if let Some(gamemode) = query.gamemode() {
        components.push(
            TextDisplayBuilder::new(format!("# Gamemode:\n{}", gamemode))
                .build()
                .into(),
        );
    }

    if let Some(software) = query.software() {
        components.push(
            TextDisplayBuilder::new(format!("# Software:\n{}", software))
                .build()
                .into(),
        );
    }

    if let Some(plugins) = query.plugins() {
        components.push(
            TextDisplayBuilder::new(format!("# Plugins:\n{}", plugins.join("\n")))
                .build()
                .into(),
        )
    }

    if let Some(mods) = query.mods() {
        components.push(
            TextDisplayBuilder::new(format!("# Mods:\n{}", mods.join("\n")))
                .build()
                .into(),
        )
    }

    Ok(components)
}

pub async fn update_header(
    message_id: Id<MessageMarker>,
    channel_id: Id<ChannelMarker>,
    description: String,
    image: Option<Vec<u8>>,
    client: &twilight_http::Client,
) -> Result<()> {
    let message = client
        .message(channel_id, message_id)
        .await?
        .model()
        .await?;
    let icon = if let Some(i) = image {
        i
    } else {
        DEFAULT_ICON.to_vec()
    };
    let filename = format!("{}.png", uuid::Uuid::new_v4());
    let mut attachment = Attachment::from_bytes(filename.clone(), icon, 1);
    attachment.description("Server Favicon".to_string());
    let mediaitem = UnfurledMediaItem {
        url: format!("attachment://{}", filename),
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
    let mut components = message.components;
    components[1] = header.into();
    client
        .update_message(channel_id, message_id)
        .components(Some(&components))
        .attachments(&[attachment])
        .await?;
    Ok(())
}
