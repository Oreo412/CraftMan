use serenity::builder::*;
use serenity::model::{application, prelude::*};
use serenity::prelude::*;
use twilight_http::Client;
use twilight_http::client::InteractionClient;
use twilight_model::channel::message::component;
use twilight_model::http::interaction::*;
use twilight_model::id::Id;

pub async fn run(
    client: &twilight_http::Client,
    serenity_interaction: serenity::model::application::CommandInteraction,
) -> Result<(), serenity::Error> {
    println!("received");
    let mut responsetest = InteractionResponseData::default();
    responsetest.content = Some("Serenity to Twilight ????".to_string());
    let response = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(responsetest),
    };

    si2tr(client, &serenity_interaction, &response).await;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("twilighttest").description("test twilight response")
}

pub async fn si2tr<'a>(
    client: &'a twilight_http::Client,
    interaction: &'a serenity::model::application::CommandInteraction,
    response: &'a InteractionResponse,
) {
    println!("Sending response via twilight");
    let application_id = Id::new(interaction.application_id.get());
    let interaction_id = Id::new(interaction.id.get());
    let _result = client
        .interaction(application_id)
        .create_response(interaction_id, &interaction.token, response)
        .into_future()
        .await;
}
