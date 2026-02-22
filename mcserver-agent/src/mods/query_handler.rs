use anyhow::{Result, anyhow, bail};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use futures_util::{
    sink::{Sink, SinkExt},
    stream::{SplitSink, SplitStream, StreamExt},
};
use protocol::query_options::QueryOptions;
use protocol::query_options::QuerySend;
use protocol::serveractions::ServerActions;
use rust_mc_status::JavaStatus;
use rust_mc_status::McClient;
use rust_mc_status::ServerData;
use rust_mc_status::error::McError;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use uuid::Uuid;

pub struct QueryHandler {
    client: McClient,
    message_id: u64,
    channel_id: u64,
    port: u32,
    options: QueryOptions,
}

impl QueryHandler {
    pub fn new(port: u32, message_id: u64, channel_id: u64, options: QueryOptions) -> Self {
        QueryHandler {
            client: McClient::new(),
            message_id,
            channel_id,
            port,
            options,
        }
    }

    pub async fn ping(&self) -> Result<JavaStatus, McError> {
        if let ServerData::Java(javastatus) = self
            .client
            .ping_java(&format!("localhost:{}", self.port))
            .await?
            .data
        {
            Ok(javastatus)
        } else {
            Err(McError::InvalidResponse(
                "The returned data in the ping function is not a java status. I don't think this is supposed to be possible".to_string()
            ))
        }
    }

    pub async fn respond<S>(&mut self, sender: &mut S, request_id: Uuid) -> Result<()>
    where
        S: Sink<Message> + Unpin,
        S::Error: std::error::Error + Send + Sync + 'static,
    {
        let status = self.ping().await?;
        let Some(image_base64) = status.favicon else {
            bail!("No image found");
        };

        let description = status.description;

        println!("Boutta try to decode the image");
        let image = STANDARD.decode(
            image_base64
                .strip_prefix("data:image/png;base64,")
                .ok_or_else(|| anyhow!("Couldn't strip prefix :("))?,
        )?;
        println!("decoded image");

        let mut query_response = QuerySend::default();
        if self.options.version() {
            println!("set version");
            query_response.set_version(status.version.name);
        }

        if self.options.player_count() {
            println!("set player count");
            query_response
                .set_player_count(format!("{}/{}", status.players.online, status.players.max));
        }

        if self.options.player_list() {
            if let Some(players) = status.players.sample {
                println!("set player list");
                query_response
                    .set_player_list(players.into_iter().map(|player| player.name).collect())
            }
        }

        if self.options.map() {
            println!("set map");
            query_response.set_map(status.map);
        }

        if self.options.gamemode() {
            println!(
                "set gamemode to {}",
                status
                    .gamemode
                    .as_ref()
                    .unwrap_or(&"No gamemode found".to_string())
            );
            query_response.set_gamemode(status.gamemode);
        }

        if self.options.software() {
            println!("set software");
            query_response.set_software(status.software);
        }

        if self.options.plugins() {
            println!("set plugins");
            if let Some(plugins) = status.plugins {
                query_response.set_plugins(Some(
                    plugins.into_iter().map(|plugin| plugin.name).collect(),
                ));
            }
        }

        if self.options.mods() {
            println!("set mods");
            if let Some(mods) = status.mods {
                query_response.set_mods(Some(mods.into_iter().map(|mcmod| mcmod.modid).collect()));
            }
        }

        sender
            .send(Message::Text(
                serde_json::to_string(&ServerActions::QueryResponse(
                    request_id,
                    description,
                    image,
                    query_response,
                ))?
                .into(),
            ))
            .await?;
        Ok(())
    }
}
