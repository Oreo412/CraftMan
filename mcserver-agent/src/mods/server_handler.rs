use crate::mods::configs::Configs;
use crate::mods::query_handler::QueryHandler;
use crate::mods::server_process::ServerProcess;
use crate::mods::server_properties::ServerProperties;
use anyhow::{Result, anyhow, bail};
use protocol::query_options::QueryOptions;
use protocol::server_commands::ServerCommands;
use protocol::serveractions::ServerActions;
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use tokio::time;
use uuid::Uuid;

pub struct ServerHandler {
    config: Configs,
    pub properties: Option<ServerProperties>,
    process: Option<ServerProcess>,
    query_channel: Option<oneshot::Sender<()>>,
}

impl ServerHandler {
    pub fn new(config: Configs) -> Self {
        let properties = match ServerProperties::new(&config.dir) {
            Ok(prop) => Some(prop),
            Err(e) => panic!("{}", e),
        };
        Self {
            properties,
            config,
            process: None,
            query_channel: None,
        }
    }
    pub fn start_server(&mut self, ws_sender: UnboundedSender<ServerActions>) -> Result<()> {
        self.process = Some(ServerProcess::new(
            self.config.xms,
            self.config.xmx,
            &self.config.jar,
            &self.config.dir,
            ws_sender,
        )?);
        tracing::info!("Started server");
        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<()> {
        if let Some(process) = self.process.take() {
            process.shutdown()?;
        }
        Ok(())
    }

    pub fn update_properties(&mut self) -> &Self {
        let path_str = format!("{}/server.properties", self.config.dir);
        let path = Path::new(&path_str);

        if path.exists() {
            match self.properties.as_mut() {
                Some(props) if props.dir == self.config.dir => {
                    if let Err(e) = props.update() {
                        tracing::info!("Failed to update server properties: {}", e);
                    }
                }
                _ => {
                    self.properties = ServerProperties::new(&self.config.dir).ok();
                }
            }
        } else {
            self.properties = None;
        }

        self
    }

    pub fn get_property(&mut self, property: &str) -> Result<&str> {
        Ok(self
            .properties
            .as_mut()
            .ok_or_else(|| anyhow!("Properties not found"))?
            .get(property)
            .ok_or_else(|| anyhow!("{} not found in properties", property))?)
    }

    pub fn set(&mut self, property: &str, value: &str) -> Result<()> {
        self.properties
            .as_mut()
            .ok_or_else(|| anyhow!("Properties not found"))?
            .set(property, value)
    }
    pub async fn send_properties_response(
        &mut self,
        sender: UnboundedSender<ServerActions>,
        uuid: Uuid,
    ) -> Result<()> {
        self.properties
            .as_mut()
            .ok_or_else(|| anyhow!("Properties not found"))?
            .send_response(sender, uuid)
            .await?;
        Ok(())
    }

    pub async fn start_query(
        &mut self,
        options: QueryOptions,
        sender: UnboundedSender<ServerActions>,
        request_id: Uuid,
    ) -> Result<()> {
        self.update_properties();
        let Some(props) = &self.properties else {
            bail!("No properties for this server");
        };

        let mut query_handler = QueryHandler::new(
            props
                .get("server-port")
                .ok_or_else(|| anyhow!("No server port found"))?
                .parse::<u32>()?,
            options,
        );

        query_handler.respond(sender.clone(), request_id).await?;

        let (c_sender, c_receiver) = oneshot::channel();
        self.query_channel = Some(c_sender);
        tokio::spawn(query_loop(query_handler, c_receiver, sender));
        Ok(())
    }

    pub fn stop_query(&mut self) {
        if let Some(sender) = self.query_channel.take() {
            let _ = sender.send(());
        }
    }
    pub fn start_chat(&self) -> Result<()> {
        if let Some(process) = &self.process {
            process.set_chat(true)
        } else {
            bail!("No running process")
        }
    }

    pub fn stop_chat(&self) -> Result<()> {
        if let Some(process) = &self.process {
            process.set_chat(false)
        } else {
            bail!("No running process")
        }
    }

    pub fn send_command(&self, command: ServerCommands) -> Result<()> {
        if let Some(process) = &self.process {
            process.send_command(command)
        } else {
            bail!("No running process")
        }
    }

    pub fn id(&self) -> Uuid {
        self.config.id
    }

    pub fn config(&self) -> Configs {
        self.config.clone()
    }

    pub fn edit_config(&mut self, config: Configs) -> Result<()> {
        self.config = config;
        Ok(())
    }
}

async fn query_loop(
    mut query_handler: QueryHandler,
    mut receiver: oneshot::Receiver<()>,
    sender: UnboundedSender<ServerActions>,
) -> Result<()> {
    let mut interval = time::interval(Duration::from_secs(10));

    loop {
        tracing::info!("Updating query monitor");
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = query_handler.update(sender.clone()).await {
                    tracing::info!("Query update failed: {e}");
                }
            }

            _ = &mut receiver => {
                // Shutdown signal received
                break;
            }
        }
    }
    tracing::info!("Exiting update loop");

    Ok(())
}
