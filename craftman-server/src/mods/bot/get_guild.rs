use anyhow::{Result, bail};
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};

pub async fn get_guild(ctx: &Context, interaction: &CommandInteraction) -> Result<u64> {
    if let Some(guild_id) = interaction.guild_id {
        Ok(guild_id.get())
    } else {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Command not available outside of Discord server"),
                ),
            )
            .await?;
        bail!("Interaction outside of server")
    }
}
