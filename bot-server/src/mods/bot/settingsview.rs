use crate::appstate::AppState;
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
use twilight_util::builder::message::*;

pub async fn run(
    client: &twilight_http::Client,
    serenity_interaction: serenity::model::application::CommandInteraction,
    appstate: &AppState,
) -> Result<(), Box<dyn Error>> {
    println!("received");
    let id = serenity_interaction
        .data
        .options
        .iter()
        .find(|option| option.name == "name")
        .ok_or_else(|| "No id".to_string())?
        .value
        .as_str()
        .ok_or_else(|| "No id".to_string())?;
    let agent = appstate
        .find_connection(id)
        .await
        .ok_or_else(|| "Agent Not Found".to_string())?;

    let props = agent.request_props().await?;
    println!("Received properties from agent: {:?}", props);
    let mut responsetest = InteractionResponseData::default();
    responsetest.components = Some(build_settings_view(&props, id));
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

pub fn register() -> CreateCommand {
    let id = CreateCommandOption::new(CommandOptionType::String, "name", "Name of socket");
    CreateCommand::new("serverproperties")
        .description("edit server properties")
        .add_option(id)
}

pub fn allow_flight(allow: bool, id: &str) -> Section {
    let displaytext = TextDisplay {
        id: None,
        content: "Allow Flight".to_string(),
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

pub fn build_settings_view(props: &HashMap<String, String>, id: &str) -> Vec<Component> {
    let allow_flight_value = props
        .get("allow-flight")
        .unwrap_or(&"false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    vec![Component::Section(allow_flight(allow_flight_value, id))]
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

pub async fn update_settings_view(
    client: &twilight_http::Client,
    interaction: &serenity::model::application::ComponentInteraction,
    props: &HashMap<String, String>,
    id: &str,
) -> Result<(), Box<dyn Error>> {
    let components = build_settings_view(props, id);

    client
        .update_message(
            Id::new(interaction.channel_id.get()),
            Id::new(interaction.message.id.get()),
        )
        .components(Some(&components))
        .await?;
    Ok(())
}
