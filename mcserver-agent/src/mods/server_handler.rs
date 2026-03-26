use crate::mods::propreader::ServerProperties;
use crate::mods::query_handler::QueryHandler;
use crate::mods::server_process::ServerProcess;
use anyhow::{Result, anyhow, bail};
use protocol::query_options::QueryOptions;
use protocol::serveractions::ServerActions;
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use tokio::time;
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;

pub struct ServerHandler {
    xms: u32,
    xmx: u32,
    dir: String,
    jar: String,
    pub properties: Option<ServerProperties>,
    process: Option<ServerProcess>,
    query_channel: Option<oneshot::Sender<()>>,
}

impl Default for ServerHandler {
    fn default() -> Self {
        let props = if Path::new("./server.properties").exists() {
            ServerProperties::new("./").ok()
        } else {
            None
        };
        Self {
            xms: 1024,
            xmx: 1024,
            dir: String::from("./"),
            jar: String::from("server.jar"),
            properties: props,
            process: None,
            query_channel: None,
        }
    }
}
impl ServerHandler {
    pub fn start_server(&mut self, ws_sender: UnboundedSender<ServerActions>) -> Result<()> {
        self.process = Some(ServerProcess::new(
            self.xms, self.xmx, &self.jar, &self.dir, ws_sender,
        )?);
        println!("Started server");
        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<()> {
        if let Some(process) = self.process.take() {
            process.shutdown()?;
        }
        Ok(())
    }

    pub fn xms(mut self, xms: u32) -> Self {
        self.xms = xms;
        self
    }
    pub fn xmx(mut self, xmx: u32) -> Self {
        self.xmx = xmx;
        self
    }
    pub fn dir(mut self, dir: String) -> Self {
        self.dir = dir;
        self
    }
    pub fn jar(mut self, jar: String) -> Self {
        self.jar = jar;
        self
    }
    pub fn update_properties(&mut self) -> &Self {
        let path_str = format!("{}/server.properties", self.dir);
        let path = Path::new(&path_str);

        if path.exists() {
            match self.properties.as_mut() {
                Some(props) if props.dir == self.dir => {
                    if let Err(e) = props.update() {
                        println!("Failed to update server properties: {}", e);
                    }
                }
                _ => {
                    self.properties = ServerProperties::new(&self.dir).ok();
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
        message_id: u64,
        channel_id: u64,
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
            message_id,
            channel_id,
            options,
        );

        query_handler.respond(sender.clone(), request_id).await?;

        let (c_sender, c_receiver) = oneshot::channel();
        self.query_channel = Some(c_sender);
        tokio::spawn(query_loop(query_handler, c_receiver, sender));
        Ok(())
    }

    pub fn shutdown_query(&mut self) {
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
}

async fn query_loop(
    mut query_handler: QueryHandler,
    mut receiver: oneshot::Receiver<()>,
    sender: UnboundedSender<ServerActions>,
) -> Result<()> {
    let mut interval = time::interval(Duration::from_secs(10));

    loop {
        println!("Updating???");
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = query_handler.update(sender.clone()).await {
                    eprintln!("Query update failed: {e}");
                }
            }

            _ = &mut receiver => {
                // Shutdown signal received
                break;
            }
        }
    }
    println!("Exiting update loop");

    Ok(())
}
