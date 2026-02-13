use futures_util::Stream;
use futures_util::{
    sink::SinkExt,
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

use anyhow::Result;
use protocol::properties::property;

use crate::mods::svstarter;

pub async fn listen<R, S>(receiver: &mut R, sender: &mut S) -> Result<()>
where
    R: Stream<Item = Result<Message, Error>> + Unpin,
    S: SinkExt<Message> + Unpin,
{
    let mut process = svstarter::ServerProcess::default().dir(String::from("/home/oreo/mcserver/"));
    while let Some(msg) = receiver.next().await {
        if let Message::Text(text) = msg.unwrap() {
            if let Ok(message) = serde_json::from_str::<AgentActions>(text.as_str()) {
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
                            if let Err(e) = props.send_response(sender, request_id).await {
                                println!("Error sending properties response");
                            } else {
                                println!("Properties response sent successfully");
                            }
                        } else {
                            println!("Failed to retrieve server properties");
                        }
                    }
                    AgentActions::edit_prop(request_id, prop) => match prop {
                        property::allow_flight => {
                            let props = process
                                .properties
                                .as_mut()
                                .ok_or_else(|| anyhow::anyhow!("properties not found"))?;

                            let current = props
                                .get("allow-flight")
                                .ok_or_else(|| anyhow::anyhow!("allow-flight key not found"))?
                                .parse::<bool>()?;

                            if let Err(e) = props.set("allow-flight", &(!current).to_string()) {
                                println!("failed to set allow-flight: {}", e)
                            }
                            if let Err(e) = props.send_response(sender, request_id).await {
                                println!("Error sending properties response");
                            } else {
                                println!("Properties response sent successfully");
                            }
                        }
                    },
                    _ => {
                        println!("Received unhandled action:");
                    }
                }
            }
        }
    }
    Ok(())
}
