use futures_util::Stream;
use futures_util::{
    sink::{Sink, SinkExt},
    stream::{SplitSink, SplitStream, StreamExt},
};
use protocol::agentactions::AgentActions;
use protocol::query_options::QueryOptions;
use protocol::serveractions::ServerActions;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use anyhow::{Result, anyhow};
use protocol::properties::property;

use crate::mods::{query_handler::QueryHandler, svstarter};

pub async fn listen<R, S>(receiver: &mut R, sender: &mut S) -> Result<()>
where
    R: Stream<Item = Result<Message, Error>> + Unpin,
    S: Sink<Message> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let mut process = svstarter::ServerProcess::default().dir(String::from("/home/oreo/mcserver/"));
    while let Some(msg) = receiver.next().await {
        let Message::Text(text) = msg? else {
            return Ok(());
        };
        let message = serde_json::from_str::<AgentActions>(text.as_str())?;
        match message {
            AgentActions::message(content) => {
                println!("Received message action: {}", content);
            }
            AgentActions::sv_start => {
                process.start_server()?;
            }
            AgentActions::sv_stop => {
                process.stop_server()?;
            }
            AgentActions::StartQuery(request_id, options, message_id, channel_id) => {
                println!("Received query");
                if let Err(e) = QueryHandler::new(25565, message_id, channel_id, options)
                    .respond(sender, request_id)
                    .await
                {
                    println!("Error starting query handling: {}", e);
                }
            }
            AgentActions::request_props(request_id) => {
                println!("Received request_props action with ID: {}", request_id);
                // Here you would gather the properties and send them back to the agent
                let props = process
                    .update_properties()
                    .properties
                    .as_ref()
                    .ok_or_else(|| anyhow!("No properties in the process"))?;
                props.send_response(sender, request_id).await?;
                println!("Properties response sent successfully");
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
                    process.set("difficulty", new_difficulty)?;
                    process.send_response(sender, request_id).await?;
                }
                property::hardcore => {
                    let current = process.get_property("hardcore")?.parse::<bool>()?;

                    process.set("hardcore", &(!current).to_string())?;
                    process.send_response(sender, request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::whitelist => {
                    let current = process.get_property("white-list")?.parse::<bool>()?;

                    process.set("white-list", &(!current).to_string())?;
                    process.send_response(sender, request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::pvp => {
                    let current = process.get_property("pvp")?.parse::<bool>()?;

                    process.set("pvp", &(!current).to_string())?;
                    process.send_response(sender, request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::generate_structures => {
                    let current = process
                        .get_property("generate-structures")?
                        .parse::<bool>()?;

                    process.set("generate-structures", &(!current).to_string())?;
                    process.send_response(sender, request_id).await?;
                    println!("Properties response sent successfully");
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
                    process.set("gamemode", new_gamemode)?;
                    process.send_response(sender, request_id).await?;
                }
                property::motd(data) => {
                    process.set("motd", &data)?;
                    process.send_response(sender, request_id).await?;
                }
                property::max_players(data) => {
                    process.set("max-players", &data.to_string())?;
                    process.send_response(sender, request_id).await?;
                }
                property::max_world_size(data) => {
                    process.set("max-world-size", &data.to_string())?;
                    process.send_response(sender, request_id).await?;
                }
                property::allow_nether => {
                    let current = process.get_property("allow-nether")?.parse::<bool>()?;

                    process.set("allow-nether", &(!current).to_string())?;
                    process.send_response(sender, request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::spawn_npcs => {
                    let current = process.get_property("spawn-npcs")?.parse::<bool>()?;

                    process.set("spawn-npcs", &(!current).to_string())?;
                    process.send_response(sender, request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::spawn_animals => {
                    let current = process.get_property("spawn-animals")?.parse::<bool>()?;

                    process.set("spawn-animals", &(!current).to_string())?;
                    process.send_response(sender, request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::spawn_monsters => {
                    let current = process.get_property("spawn-monsters")?.parse::<bool>()?;

                    process.set("spawn-monsters", &(!current).to_string())?;
                    process.send_response(sender, request_id).await?;
                    println!("Properties response sent successfully");
                }
                property::view_distance(data) => {
                    process.set("view-distance", &data.to_string())?;
                    process.send_response(sender, request_id).await?;
                }
                property::simulation_distance(data) => {
                    process.set("simulation-distance", &data.to_string())?;
                    process.send_response(sender, request_id).await?;
                }
                property::spawn_protection(data) => {
                    process.set("spawn-protection", &data.to_string())?;
                    process.send_response(sender, request_id).await?;
                }
            },
            _ => {
                println!("Received unhandled action:");
            }
        }
    }
    Ok(())
}
