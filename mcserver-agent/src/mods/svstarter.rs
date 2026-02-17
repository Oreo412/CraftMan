use crate::mods::propreader::ServerProperties;
use anyhow::{Result, anyhow, bail};
use futures_util::SinkExt;
use std::path::Path;
use std::process::*;
use std::{io::*, path};
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
            .spawn()?;
        self.child = Some(child);
        self.update_properties();
        Ok(())
    }

    pub fn stop_server(&mut self) -> Result<()> {
        let child = &mut self
            .child
            .as_mut()
            .ok_or_else(|| anyhow!("No child process found"))?;
        if let Some(stdin) = &mut child.stdin {
            writeln!(stdin, "stop")?;
        }
        let _ = child.wait();
        self.child = None;
        self.update_properties();
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
            .ok_or_else(|| anyhow::anyhow!("Properties not found"))?
            .get(property)
            .ok_or_else(|| anyhow::anyhow!("{} not found in properties", property))?)
    }

    pub fn set(&mut self, property: &str, value: &str) -> Result<()> {
        Ok(self
            .properties
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Properties not found"))?
            .set(property, value)?)
    }
    pub async fn send_response<S>(&mut self, sender: &mut S, uuid: Uuid) -> Result<()>
    where
        S: SinkExt<Message> + Unpin,
        S::Error: std::error::Error + Send + Sync + 'static,
    {
        self.properties
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Properties not found"))?
            .send_response(sender, uuid)
            .await?;
        Ok(())
    }
}
