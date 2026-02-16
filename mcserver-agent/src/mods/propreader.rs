use anyhow::{Result, anyhow, bail};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use protocol::serveractions::ServerActions;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Seek, SeekFrom};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use uuid::Uuid;

pub struct ServerProperties {
    properties: HashMap<String, String>,
    pub dir: String,
}

impl ServerProperties {
    pub fn new(dir: &str) -> Result<Self, Box<dyn Error>> {
        let reader = BufReader::new(File::open(format!("{}/server.properties", dir))?);
        let mut properties = default_properties();
        properties.extend(java_properties::read(reader)?);
        Ok(ServerProperties {
            properties,
            dir: dir.to_string(),
        })
    }
    pub fn get(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        if let None = self.properties.get(key) {
            bail!("Key {} not found", key);
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .open(format!("{}/server.properties", self.dir))?;

        {
            let mut temp = self.properties.clone();
            temp.insert(key.to_string(), value.to_string());
            let mut writer = BufWriter::new(&file);
            java_properties::write(&mut writer, &temp)?
        }

        file.seek(SeekFrom::Start(0))?;
        let reader = BufReader::new(file);
        self.properties = java_properties::read(reader)?;
        Ok(())
    }
    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let reader = BufReader::new(File::open(format!("{}/server.properties", self.dir))?);
        self.properties = java_properties::read(reader)?;
        Ok(())
    }
    pub async fn send_update<S>(
        &mut self,
        sender: &mut S,
    ) -> Result<(), <S as futures_util::Sink<Message>>::Error>
    where
        S: SinkExt<Message> + Unpin,
    {
        sender
            .send(Message::Text(
                serde_json::to_string(&ServerActions::props_update(self.properties.clone()))
                    .unwrap()
                    .into(),
            ))
            .await
    }
    pub async fn send_response<S>(
        &self,
        sender: &mut S,
        uuid: Uuid,
    ) -> Result<(), <S as futures_util::Sink<Message>>::Error>
    where
        S: SinkExt<Message> + Unpin,
    {
        sender
            .send(Message::Text(
                serde_json::to_string(&ServerActions::response_props(
                    uuid,
                    self.properties.clone(),
                ))
                .unwrap()
                .into(),
            ))
            .await
    }
}

fn default_properties() -> HashMap<String, String> {
    HashMap::from([
        ("enable-jmx-monitoring".into(), "false".into()),
        ("rcon.port".into(), "25575".into()),
        ("level-seed".into(), "".into()),
        ("gamemode".into(), "survival".into()),
        ("enable-command-block".into(), "false".into()),
        ("enable-query".into(), "false".into()),
        ("generator-settings".into(), "{}".into()),
        ("enforce-secure-profile".into(), "true".into()),
        ("level-name".into(), "world".into()),
        ("motd".into(), "A Minecraft Server".into()),
        ("query.port".into(), "25565".into()),
        ("pvp".into(), "true".into()),
        ("generate-structures".into(), "true".into()),
        ("max-chained-neighbor-updates".into(), "1000000".into()),
        ("difficulty".into(), "easy".into()),
        ("network-compression-threshold".into(), "256".into()),
        ("max-tick-time".into(), "60000".into()),
        ("require-resource-pack".into(), "false".into()),
        ("use-native-transport".into(), "true".into()),
        ("max-players".into(), "20".into()),
        ("online-mode".into(), "true".into()),
        ("enable-status".into(), "true".into()),
        ("allow-flight".into(), "false".into()),
        ("initial-disabled-packs".into(), "".into()),
        ("broadcast-rcon-to-ops".into(), "true".into()),
        ("view-distance".into(), "10".into()),
        ("server-ip".into(), "".into()),
        ("resource-pack-prompt".into(), "".into()),
        ("allow-nether".into(), "true".into()),
        ("server-port".into(), "25565".into()),
        ("enable-rcon".into(), "false".into()),
        ("sync-chunk-writes".into(), "true".into()),
        ("op-permission-level".into(), "4".into()),
        ("prevent-proxy-connections".into(), "false".into()),
        ("hide-online-players".into(), "false".into()),
        ("resource-pack".into(), "".into()),
        ("entity-broadcast-range-percentage".into(), "100".into()),
        ("simulation-distance".into(), "10".into()),
        ("rcon.password".into(), "".into()),
        ("player-idle-timeout".into(), "0".into()),
        ("force-gamemode".into(), "false".into()),
        ("rate-limit".into(), "0".into()),
        ("hardcore".into(), "false".into()),
        ("white-list".into(), "false".into()),
        ("broadcast-console-to-ops".into(), "true".into()),
        ("spawn-npcs".into(), "true".into()),
        ("spawn-animals".into(), "true".into()),
        ("log-ips".into(), "true".into()),
        ("function-permission-level".into(), "2".into()),
        ("initial-enabled-packs".into(), "vanilla".into()),
        ("level-type".into(), "minecraft:normal".into()),
        ("text-filtering-config".into(), "".into()),
        ("spawn-monsters".into(), "true".into()),
        ("enforce-whitelist".into(), "false".into()),
        ("spawn-protection".into(), "16".into()),
        ("resource-pack-sha1".into(), "".into()),
        ("max-world-size".into(), "29999984".into()),
    ])
}
