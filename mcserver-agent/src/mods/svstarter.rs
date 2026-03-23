use crate::mods::propreader::ServerProperties;
use crate::mods::query_handler::QueryHandler;
use anyhow::{Result, anyhow, bail};
use futures_util::Sink;
use futures_util::SinkExt;
use protocol::chat::SendChat;
use protocol::query_options::QueryOptions;
use protocol::serveractions::ServerActions;
use std::path::Path;
use std::process::*;
use std::sync::mpsc;
use std::time::Duration;
use std::{io::*, path};
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::Lines;
use tokio::process::Child;
use tokio::process::ChildStdout;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use tokio::time;
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;

pub fn stop_server(mut stdin: ChildStdin) {
    let _result = writeln!(stdin, "stop");
}

pub struct ServerProcess {
    xms: u32,
    xmx: u32,
    dir: String,
    jar: String,
    pub properties: Option<ServerProperties>,
    child: Option<Child>,
    query_channel: Option<oneshot::Sender<()>>,
    chat_channel_id: Mutex<Option<u64>>, //As stdioreader will be run in a separate spawned task
                                         //from the listener, listener could try to change it at the
                                         //same time as it's read by stdioreader
}

impl Default for ServerProcess {
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
            child: None,
            query_channel: None,
            chat_channel_id: Mutex::new(None),
        }
    }
}
impl ServerProcess {
    pub fn start_server(&mut self) -> Result<()> {
        let child = Command::new("java")
            .current_dir(&self.dir)
            .arg(format!("-Xmx{}M", self.xmx))
            .arg(format!("-Xms{}M", self.xms))
            .arg("-jar")
            .arg(&self.jar)
            .arg("nogui")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        self.child = Some(child);
        self.update_properties();
        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<()> {
        let child = &mut self
            .child
            .as_mut()
            .ok_or_else(|| anyhow!("No child process found"))?;
        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(b"stop\n").await?;
        }
        let _ = child.wait();
        self.child = None;
        self.update_properties();
        Ok(())
    }

    pub async fn stdio_reader(&mut self, sender: UnboundedSender<Message>) -> Result<()> {
        let channel_id = if let Some(id) = *self.chat_channel_id.lock().await {
            id
        } else {
            return Ok(());
        };
        let stdout = self
            .child
            .as_mut()
            .ok_or_else(|| anyhow!("no child process found"))?
            .stdout
            .take()
            .ok_or_else(|| anyhow!("No stdout found"))?;

        let lines = BufReader::new(stdout).lines();
        tokio::spawn(chat_listener(lines, channel_id, sender));

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
    pub async fn channel(&mut self, channel_id: Option<u64>) {
        *self.chat_channel_id.lock().await = channel_id
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
        sender: UnboundedSender<Message>,
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
        sender: UnboundedSender<Message>,
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
}

async fn query_loop(
    mut query_handler: QueryHandler,
    mut receiver: oneshot::Receiver<()>,
    sender: UnboundedSender<Message>,
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

async fn chat_listener(
    mut lines: Lines<BufReader<ChildStdout>>,
    channel_id: u64,
    sender: UnboundedSender<Message>,
) -> Result<()> {
    while let Some(new_message) = lines.next_line().await? {
        sender.send(Message::Text(
            serde_json::to_string(&ServerActions::NewMessage(channel_id, new_message))?.into(),
        ))?
    }
    Ok(())
}
