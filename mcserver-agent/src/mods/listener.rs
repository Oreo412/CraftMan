use futures_util::Stream;
use futures_util::stream::StreamExt;
use protocol::agentactions::AgentActions;
use protocol::serveractions::ServerActions;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::tungstenite::protocol::Message;

use anyhow::{Result, anyhow};
use protocol::properties::property;

use crate::mods::server_handler::{self, ServerHandler};

pub async fn listen<R>(
    receiver: &mut R,
    sender: UnboundedSender<ServerActions>,
    handler: &mut ServerHandler,
) -> Result<()>
where
    R: Stream<Item = Result<Message, Error>> + Unpin,
{
    while let Some(msg) = receiver.next().await {
        let Message::Text(text) = msg? else {
            return Ok(());
        };
        let message = serde_json::from_str::<AgentActions>(text.as_str())?;
        match message {
            AgentActions::Message(content) => {
                println!("Received message action: {}", content);
            }
            AgentActions::SvStart(id) => {
                println!("Starting server");
                if handler.start_server(sender.clone()).is_ok() {
                    sender.send(ServerActions::StartResponse(id))?;
                }
            }
            AgentActions::SvStop(id) => {
                if handler.stop_server().await.is_ok() {
                    sender.send(ServerActions::StopResponse(id))?;
                }
            }
            AgentActions::StartQuery(request_id, options) => {
                println!("Received query");
                if let Err(e) = handler
                    .start_query(options, sender.clone(), request_id)
                    .await
                {
                    println!("Error starting query handling: {}", e);
                }
            }
            AgentActions::StopQuery => {
                handler.stop_query();
            }
            AgentActions::RequestProps(request_id) => {
                println!("Received request_props action with ID: {}", request_id);
                // Here you would gather the properties and send them back to the agent
                let props = handler
                    .update_properties()
                    .properties
                    .as_ref()
                    .ok_or_else(|| anyhow!("No properties in the process"))?;
                props.send_response(sender.clone(), request_id).await?;
                println!("Properties response sent successfully");
            }
            AgentActions::EditProp(request_id, prop) => {
                match prop {
                    property::AllowFlight => {
                        let current = handler.get_property("allow-flight")?.parse::<bool>()?;

                        handler.set("allow-flight", &(!current).to_string())?;
                        println!("Properties response sent successfully");
                    }
                    property::Difficulty => {
                        let current = handler.get_property("difficulty")?;
                        let new_difficulty = match current {
                            "peaceful" => "easy",
                            "easy" => "normal",
                            "normal" => "hard",
                            "hard" => "peaceful",
                            _ => "easy",
                        };
                        handler.set("difficulty", new_difficulty)?;
                    }
                    property::Hardcore => {
                        let current = handler.get_property("hardcore")?.parse::<bool>()?;

                        handler.set("hardcore", &(!current).to_string())?;
                        println!("Properties response sent successfully");
                    }
                    property::Whitelist => {
                        let current = handler.get_property("white-list")?.parse::<bool>()?;

                        handler.set("white-list", &(!current).to_string())?;
                        println!("Properties response sent successfully");
                    }
                    property::PVP => {
                        let current = handler.get_property("pvp")?.parse::<bool>()?;

                        handler.set("pvp", &(!current).to_string())?;
                        println!("Properties response sent successfully");
                    }
                    property::GenerateStructures => {
                        let current = handler
                            .get_property("generate-structures")?
                            .parse::<bool>()?;

                        handler.set("generate-structures", &(!current).to_string())?;
                        println!("Properties response sent successfully");
                    }
                    property::Gamemode => {
                        let current = handler.get_property("gamemode")?;
                        let new_gamemode = match current {
                            "survival" => "creative",
                            "creative" => "adventure",
                            "adventure" => "spectator",
                            "spectator" => "survival",
                            _ => "survival",
                        };
                        handler.set("gamemode", new_gamemode)?;
                    }
                    property::MOTD(data) => {
                        handler.set("motd", &data)?;
                    }
                    property::MaxPlayers(data) => {
                        handler.set("max-players", &data.to_string())?;
                    }
                    property::MaxWorldSize(data) => {
                        handler.set("max-world-size", &data.to_string())?;
                    }
                    property::AllowNether => {
                        let current = handler.get_property("allow-nether")?.parse::<bool>()?;

                        handler.set("allow-nether", &(!current).to_string())?;
                        println!("Properties response sent successfully");
                    }
                    property::SpawnNPC => {
                        let current = handler.get_property("spawn-npcs")?.parse::<bool>()?;

                        handler.set("spawn-npcs", &(!current).to_string())?;
                        println!("Properties response sent successfully");
                    }
                    property::SpawnAnimals => {
                        let current = handler.get_property("spawn-animals")?.parse::<bool>()?;

                        handler.set("spawn-animals", &(!current).to_string())?;
                        println!("Properties response sent successfully");
                    }
                    property::SpawnMonsters => {
                        let current = handler.get_property("spawn-monsters")?.parse::<bool>()?;

                        handler.set("spawn-monsters", &(!current).to_string())?;
                        println!("Properties response sent successfully");
                    }
                    property::ViewDistance(data) => {
                        handler.set("view-distance", &data.to_string())?;
                    }
                    property::SimulationDistance(data) => {
                        handler.set("simulation-distance", &data.to_string())?;
                    }
                    property::SpawnProtection(data) => {
                        handler.set("spawn-protection", &data.to_string())?;
                    }
                }
                handler
                    .send_properties_response(sender.clone(), request_id)
                    .await?;
            }
            AgentActions::StartChatStream(uuid) => {
                handler.start_chat()?;
                println!("Sending start chat response");
                sender.send(ServerActions::StartChatResponse(uuid))?;
            }
            AgentActions::StopChatStream(uuid) => {
                handler.stop_chat()?;
                println!("Sending stop chat response");
                sender.send(ServerActions::StopChatResponse(uuid))?;
            }
            AgentActions::ConnectionKey(key) => {
                println!("Enter key into discord: {}", key);
            }
        }
    }
    Ok(())
}
