use serenity::model::application::CommandInteraction;
use twilight_http::Client;
use twilight_model::http::interaction::InteractionResponse;
use twilight_model::id::Id;

pub async fn si2tr<'a>(
    //Converts Sirenity interactioin to a Twilight response
    client: &'a Client,
    interaction: &'a CommandInteraction,
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
