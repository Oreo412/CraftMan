use futures_util::Stream;
use futures_util::{
    sink::{Sink, SinkExt},
    stream::{SplitSink, SplitStream, StreamExt},
};
use protocol::agentactions::AgentActions;
use protocol::serveractions::ServerActions;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use anyhow::{Result, anyhow};
use protocol::properties::property;

use crate::mods::svstarter;

pub async fn listen<R, S>(receiver: &mut R, sender: &mut S) -> Result<()>
where
    R: Stream<Item = Result<Message, Error>> + Unpin,
    S: Sink<Message> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let mut process = svstarter::ServerProcess::default().dir(String::from("/home/oreo/mcserver/"));
    while let Some(msg) = receiver.next().await {
        if let Message::Text(text) = msg? {
            let message = serde_json::from_str::<AgentActions>(text.as_str())?;
            match message {
                AgentActions::message(content) => {
                    println!("Received message action: {}", content);
                }
                AgentActions::sv_start => {
                    if let Err(e) = process.start_server() {
                        println!("Error starting server: {}", e);
                    } else {
                        println!("Server started successfully");
                    }
                }
                AgentActions::sv_stop => {
                    if let Err(e) = process.stop_server() {
                        println!("Error stopping server: {}", e);
                    } else {
                        println!("Server stopped successfully");
                    }
                }
                AgentActions::request_props(request_id) => {
                    println!("Received request_props action with ID: {}", request_id);
                    // Here you would gather the properties and send them back to the agent
                    if let Some(props) = &process.update_properties().properties {
                        props.send_response(sender, request_id).await?;
                        println!("Properties response sent successfully");
                    } else {
                        println!("Failed to retrieve server properties");
                    }
                }
                AgentActions::edit_prop(request_id, prop) => match prop {
                    property::allow_flight => {
                        let current = process.get_property("allow-flight")?.parse::<bool>()?;

                        process.set("allow-flight", &(!current).to_string())?;
                        process.send_response(sender, request_id).await?;
                        println!("Properties response sent successfully");
                    }
                    property::difficulty => {
                        let current = process.get_property("difficulty")?;
                        let new_difficulty = match current {
                            "peaceful" => "easy",
                            "easy" => "normal",
                            "normal" => "hard",
                            "hard" => "peaceful",
                            _ => "easy",
                        };
                        if let Err(e) = process.set("difficulty", new_difficulty) {
                            println!("Failed to set difficulty: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response")
                        }
                    }
                    property::hardcore => {
                        let current = process.get_property("hardcore")?.parse::<bool>()?;

                        if let Err(e) = process.set("hardcore", &(!current).to_string()) {
                            println!("failed to set hardcore: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response");
                        } else {
                            println!("Properties response sent successfully");
                        }
                    }
                    property::whitelist => {
                        let current = process.get_property("white-list")?.parse::<bool>()?;

                        if let Err(e) = process.set("white-list", &(!current).to_string()) {
                            println!("failed to set whitelist: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response");
                        } else {
                            println!("Properties response sent successfully");
                        }
                    }
                    property::pvp => {
                        let current = process.get_property("pvp")?.parse::<bool>()?;

                        if let Err(e) = process.set("pvp", &(!current).to_string()) {
                            println!("failed to set pvp: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response");
                        } else {
                            println!("Properties response sent successfully");
                        }
                    }
                    property::generate_structures => {
                        let current = process
                            .get_property("generate-structures")?
                            .parse::<bool>()?;

                        if let Err(e) = process.set("generate-structures", &(!current).to_string())
                        {
                            println!("failed to set generate-structures: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response");
                        } else {
                            println!("Properties response sent successfully");
                        }
                    }
                    property::gamemode => {
                        let current = process.get_property("gamemode")?;
                        let new_gamemode = match current {
                            "survival" => "creative",
                            "creative" => "adventure",
                            "adventure" => "spectator",
                            "spectator" => "survival",
                            _ => "survival",
                        };
                        if let Err(e) = process.set("gamemode", new_gamemode) {
                            println!("Failed to set gamemode: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response")
                        }
                    }
                    property::motd(data) => {
                        if let Err(e) = process.set("motd", &data) {
                            println!("Failed to set gamemode: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response")
                        }
                    }
                    property::max_players(data) => {
                        if let Err(e) = process.set("max-players", &data.to_string()) {
                            println!("Failed to set max players: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response")
                        }
                    }
                    property::max_world_size(data) => {
                        if let Err(e) = process.set("max-world-size", &data.to_string()) {
                            println!("Failed to set max world size: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response")
                        }
                    }
                    property::allow_nether => {
                        let current = process.get_property("allow-nether")?.parse::<bool>()?;

                        if let Err(e) = process.set("allow-nether", &(!current).to_string()) {
                            println!("failed to set allow-nether: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response");
                        } else {
                            println!("Properties response sent successfully");
                        }
                    }
                    property::spawn_npcs => {
                        let current = process.get_property("spawn-npcs")?.parse::<bool>()?;

                        if let Err(e) = process.set("spawn-npcs", &(!current).to_string()) {
                            println!("failed to set spawn-npcs: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response");
                        } else {
                            println!("Properties response sent successfully");
                        }
                    }
                    property::spawn_animals => {
                        let current = process.get_property("spawn-animals")?.parse::<bool>()?;

                        if let Err(e) = process.set("spawn-animals", &(!current).to_string()) {
                            println!("failed to set spawn-animals: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response");
                        } else {
                            println!("Properties response sent successfully");
                        }
                    }
                    property::spawn_monsters => {
                        let current = process.get_property("spawn-monsters")?.parse::<bool>()?;

                        if let Err(e) = process.set("spawn-monsters", &(!current).to_string()) {
                            println!("failed to set spawn-monsters: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response");
                        } else {
                            println!("Properties response sent successfully");
                        }
                    }
                    property::view_distance(data) => {
                        if let Err(e) = process.set("view-distance", &data.to_string()) {
                            println!("Failed to set view distance: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response")
                        }
                    }
                    property::simulation_distance(data) => {
                        if let Err(e) = process.set("simulation-distance", &data.to_string()) {
                            println!("Failed to set simulation distance: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response")
                        }
                    }
                    property::spawn_protection(data) => {
                        if let Err(e) = process.set("spawn-protection", &data.to_string()) {
                            println!("Failed to set spawn-protection: {}", e)
                        }
                        if let Err(e) = process.send_response(sender, request_id).await {
                            println!("Error sending properties response")
                        }
                    }
                },
                _ => {
                    println!("Received unhandled action:");
                }
            }
        }
    }
    Ok(())
}
