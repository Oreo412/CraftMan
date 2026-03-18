use futures_util::Stream;
use futures_util::stream::StreamExt;
use protocol::agentactions::AgentActions;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::tungstenite::protocol::Message;

use anyhow::{Result, anyhow};
use protocol::properties::property;

use crate::mods::svstarter;

pub async fn listen<R>(receiver: &mut R, sender: UnboundedSender<Message>) -> Result<()>
where
    R: Stream<Item = Result<Message, Error>> + Unpin,
{
    let mut process = svstarter::ServerProcess::default().dir(String::from("/home/oreo/mcserver/"));
    while let Some(msg) = receiver.next().await {
        let Message::Text(text) = msg? else {
            return Ok(());
        };
        let message = serde_json::from_str::<AgentActions>(text.as_str())?;
        match message {
            AgentActions::Message(content) => {
                println!("Received message action: {}", content);
            }
            AgentActions::SvStart => {
                process.start_server()?;
            }
            AgentActions::SvStop => {
                process.stop_server()?;
            }
            AgentActions::StartQuery(request_id, options, message_id, channel_id) => {
                println!("Received query");
                if let Err(e) = process
                    .start_query(message_id, channel_id, options, sender.clone(), request_id)
                    .await
                {
                    println!("Error starting query handling: {}", e);
                }
            }
            AgentActions::RequestProps(request_id) => {
                println!("Received request_props action with ID: {}", request_id);
                // Here you would gather the properties and send them back to the agent
                let props = process
                    .update_properties()
                    .properties
                    .as_ref()
                    .ok_or_else(|| anyhow!("No properties in the process"))?;
                props.send_response(sender.clone(), request_id).await?;
                println!("Properties response sent successfully");
            }
            AgentActions::EditProp(request_id, prop) => match prop {
                property::AllowFlight => {
                    let current = process.get_property("allow-flight")?.parse::<bool>()?;

                    process.set("allow-flight", &(!current).to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::Difficulty => {
                    let current = process.get_property("difficulty")?;
                    let new_difficulty = match current {
                        "peaceful" => "easy",
                        "easy" => "normal",
                        "normal" => "hard",
                        "hard" => "peaceful",
                        _ => "easy",
                    };
                    process.set("difficulty", new_difficulty)?;
                    process.send_response(sender.clone(), request_id).await?;
                }
                property::Hardcore => {
                    let current = process.get_property("hardcore")?.parse::<bool>()?;

                    process.set("hardcore", &(!current).to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::Whitelist => {
                    let current = process.get_property("white-list")?.parse::<bool>()?;

                    process.set("white-list", &(!current).to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::PVP => {
                    let current = process.get_property("pvp")?.parse::<bool>()?;

                    process.set("pvp", &(!current).to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::GenerateStructures => {
                    let current = process
                        .get_property("generate-structures")?
                        .parse::<bool>()?;

                    process.set("generate-structures", &(!current).to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::Gamemode => {
                    let current = process.get_property("gamemode")?;
                    let new_gamemode = match current {
                        "survival" => "creative",
                        "creative" => "adventure",
                        "adventure" => "spectator",
                        "spectator" => "survival",
                        _ => "survival",
                    };
                    process.set("gamemode", new_gamemode)?;
                    process.send_response(sender.clone(), request_id).await?;
                }
                property::MOTD(data) => {
                    process.set("motd", &data)?;
                    process.send_response(sender.clone(), request_id).await?;
                }
                property::MaxPlayers(data) => {
                    process.set("max-players", &data.to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                }
                property::MaxWorldSize(data) => {
                    process.set("max-world-size", &data.to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                }
                property::AllowNether => {
                    let current = process.get_property("allow-nether")?.parse::<bool>()?;

                    process.set("allow-nether", &(!current).to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::SpawnNPC => {
                    let current = process.get_property("spawn-npcs")?.parse::<bool>()?;

                    process.set("spawn-npcs", &(!current).to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::SpawnAnimals => {
                    let current = process.get_property("spawn-animals")?.parse::<bool>()?;

                    process.set("spawn-animals", &(!current).to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::SpawnMonsters => {
                    let current = process.get_property("spawn-monsters")?.parse::<bool>()?;

                    process.set("spawn-monsters", &(!current).to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::ViewDistance(data) => {
                    process.set("view-distance", &data.to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                }
                property::SimulationDistance(data) => {
                    process.set("simulation-distance", &data.to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                }
                property::SpawnProtection(data) => {
                    process.set("spawn-protection", &data.to_string())?;
                    process.send_response(sender.clone(), request_id).await?;
                }
            },
        }
    }
    Ok(())
}
