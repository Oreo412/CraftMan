use crate::gui::{
    gui_actions::{ConfigRequest, EditRequestReturn},
    tui::GuiEvents,
};
use futures_util::Stream;
use futures_util::stream::StreamExt;
use protocol::agentactions::AgentActions;
use protocol::serveractions::ServerActions;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::tungstenite::protocol::Message;

use anyhow::{Result, anyhow};
use protocol::properties::Property;

use crate::mods::server_handler::ServerHandler;

pub async fn listen<R>(
    mut receiver: R,
    sender: UnboundedSender<ServerActions>,
    handler: &mut ServerHandler,
    agent_receiver: &mut UnboundedReceiver<ConfigRequest>,
    agent_to_tui: UnboundedSender<GuiEvents>,
) -> Result<()>
where
    R: Stream<Item = Result<Message, Error>> + Unpin,
{
    loop {
        tokio::select! {
            next_msg = receiver.next() => {
                if let Some(msg) = next_msg {
                    if let Err(e) = websocket_action(handler, &sender, msg?, &agent_to_tui).await {
                        tracing::error!("Error handling websocket action: {}", e);
                    }
                }
                else {
                    tracing::warn!("Websocket Closed");
                    return Ok(());
                }

            }

            next_msg = agent_receiver.recv() => {
                if let Some(msg) = next_msg {
                    if let Err(e) = gui_action(handler, msg).await {
                        tracing::error!("Error handling AgentAction: {}", e);
                    }
                }
                else {
                    tracing::warn!("GUI lost");
                    return Ok(());
                }

            }


        }
    }
}

async fn websocket_action(
    handler: &mut ServerHandler,
    sender: &UnboundedSender<ServerActions>,
    msg: Message,
    agent_to_tui: &UnboundedSender<GuiEvents>,
) -> Result<()> {
    let Message::Text(text) = msg else {
        return Ok(());
    };
    let message = serde_json::from_str::<AgentActions>(text.as_str())?;
    match message {
        AgentActions::Message(content) => {
            tracing::info!("Received message action: {}", content);
        }
        AgentActions::SvStart(id) => {
            tracing::info!("Starting server");
            if handler.start_server(sender.clone()).is_ok() {
                sender.send(ServerActions::StartResponse(id))?;
                agent_to_tui.send(GuiEvents::ServerStarted)?;
            }
        }
        AgentActions::SvStop(id) => {
            if handler.stop_server().await.is_ok() {
                sender.send(ServerActions::StopResponse(id))?;
                agent_to_tui.send(GuiEvents::ServerStopped)?;
            }
        }
        AgentActions::StartQuery(request_id, options) => {
            tracing::info!("Received query");
            if let Err(e) = handler
                .start_query(options, sender.clone(), request_id)
                .await
            {
                tracing::error!("Error starting query handling: {}", e);
            }
        }
        AgentActions::StopQuery => {
            handler.stop_query();
        }
        AgentActions::RequestProps(request_id) => {
            tracing::info!("Received request_props action with ID: {}", request_id);
            let props = handler
                .update_properties()
                .properties
                .as_ref()
                .ok_or_else(|| anyhow!("No properties in the process"))?;
            props.send_response(sender.clone(), request_id).await?;
            tracing::info!("Properties response sent successfully");
        }
        AgentActions::EditProp(request_id, prop) => {
            match prop {
                Property::AllowFlight => {
                    let current = handler.get_property("allow-flight")?.parse::<bool>()?;

                    handler.set("allow-flight", &(!current).to_string())?;
                    tracing::info!("Properties response sent successfully");
                }
                Property::Difficulty => {
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
                Property::Hardcore => {
                    let current = handler.get_property("hardcore")?.parse::<bool>()?;

                    handler.set("hardcore", &(!current).to_string())?;
                    tracing::info!("Properties response sent successfully");
                }
                Property::Whitelist => {
                    let current = handler.get_property("white-list")?.parse::<bool>()?;

                    handler.set("white-list", &(!current).to_string())?;
                    tracing::info!("Properties response sent successfully");
                }
                Property::PVP => {
                    let current = handler.get_property("pvp")?.parse::<bool>()?;

                    handler.set("pvp", &(!current).to_string())?;
                    tracing::info!("Properties response sent successfully");
                }
                Property::GenerateStructures => {
                    let current = handler
                        .get_property("generate-structures")?
                        .parse::<bool>()?;

                    handler.set("generate-structures", &(!current).to_string())?;
                    tracing::info!("Properties response sent successfully");
                }
                Property::Gamemode => {
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
                Property::MOTD(data) => {
                    handler.set("motd", &data)?;
                }
                Property::MaxPlayers(data) => {
                    handler.set("max-players", &data.to_string())?;
                }
                Property::MaxWorldSize(data) => {
                    handler.set("max-world-size", &data.to_string())?;
                }
                Property::AllowNether => {
                    let current = handler.get_property("allow-nether")?.parse::<bool>()?;

                    handler.set("allow-nether", &(!current).to_string())?;
                    tracing::info!("Properties response sent successfully");
                }
                Property::SpawnNPC => {
                    let current = handler.get_property("spawn-npcs")?.parse::<bool>()?;

                    handler.set("spawn-npcs", &(!current).to_string())?;
                    tracing::info!("Properties response sent successfully");
                }
                Property::SpawnAnimals => {
                    let current = handler.get_property("spawn-animals")?.parse::<bool>()?;

                    handler.set("spawn-animals", &(!current).to_string())?;
                    tracing::info!("Properties response sent successfully");
                }
                Property::SpawnMonsters => {
                    let current = handler.get_property("spawn-monsters")?.parse::<bool>()?;

                    handler.set("spawn-monsters", &(!current).to_string())?;
                    tracing::info!("Properties response sent successfully");
                }
                Property::ViewDistance(data) => {
                    handler.set("view-distance", &data.to_string())?;
                }
                Property::SimulationDistance(data) => {
                    handler.set("simulation-distance", &data.to_string())?;
                }
                Property::SpawnProtection(data) => {
                    handler.set("spawn-protection", &data.to_string())?;
                }
            }
            handler
                .send_properties_response(sender.clone(), request_id)
                .await?;
        }
        AgentActions::StartChatStream(uuid) => {
            handler.start_chat()?;
            tracing::info!("Sending start chat response");
            sender.send(ServerActions::StartChatResponse(uuid))?;
        }
        AgentActions::StopChatStream(uuid) => {
            handler.stop_chat()?;
            tracing::info!("Sending stop chat response");
            sender.send(ServerActions::StopChatResponse(uuid))?;
        }
        AgentActions::ValidationToken(key) => {
            tracing::info!("Enter key into discord: {}", key);
            agent_to_tui.send(GuiEvents::Validate(key))?;
        }
        AgentActions::Validate => {
            agent_to_tui.send(GuiEvents::Validated)?;
        }
        AgentActions::ServerCommand(id, command) => {
            handler.send_command(command)?;
            sender.send(ServerActions::SendCommandResponse(id))?;
        }
    }
    Ok(())
}

async fn gui_action(handler: &mut ServerHandler, msg: ConfigRequest) -> anyhow::Result<()> {
    tracing::info!("Handling gui action");
    match msg {
        ConfigRequest::Request(sender) => {
            sender
                .send(handler.config())
                .map_err(|_| anyhow::anyhow!("failed to send config"))?;
        }
        ConfigRequest::Edit(sender, config) => {
            tracing::info!("Received edit request. Handling");
            if let Err(e) = handler.edit_config(config) {
                sender
                    .send(EditRequestReturn::EditInvalid(e.to_string()))
                    .map_err(|_| anyhow::anyhow!("failed to send config"))?;
            } else {
                sender
                    .send(EditRequestReturn::Edited)
                    .map_err(|_| anyhow::anyhow!("failed to send config"))?;
            };
        }
    };
    Ok(())
}
