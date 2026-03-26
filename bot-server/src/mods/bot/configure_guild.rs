use crate::appstate::AppState;
use anyhow::{Result, anyhow};
use protocol::agentactions::AgentActions;
use serenity::builder::*;
use serenity::model::application::CommandInteraction;
use uuid::Uuid;

// pub async fn start_mc_server(interaction: &CommandInteraction, appstate: &AppState) -> Result<()> {
//     let id = interaction
//         .guild_id
//         .ok_or_else(|| anyhow!("interaction took place outside of guild"))?
//         .get();
//     println!("Sending start signal to socket '{}'", id);
//     appstate
//         .send_by_guild(id, AgentActions::SvStart(Uuid::new_v4()))
//         .await?;
//
//     Ok(())
// }
//
// pub fn register() -> CreateCommand {
//     //let id= CreateCommandOption::new(CommandOptionType::String, "name", "Name of socket");
//     CreateCommand::new("configure_guild").description("Configure the guild")
// }
