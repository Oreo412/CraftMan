use anyhow::Result;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Configs {
    pub id: Uuid,
    pub xms: u32,
    pub xmx: u32,
    pub dir: String,
    pub jar: String,
}

impl Configs {
    fn path() -> PathBuf {
        let proj =
            ProjectDirs::from("dev", "oreo", "craftman").expect("Project directories missing");
        let configdir = proj.config_local_dir();

        std::fs::create_dir_all(configdir).expect("Unable to find or crate config directory");

        configdir.join("config.json")
    }

    pub fn new() -> Self {
        let config = Configs::build();
        config.save();
        config
    }

    fn build() -> Self {
        let mut path = Configs::path();

        if path.exists() {
            let data = std::fs::read_to_string(&path).expect("Failed to read config");

            serde_json::from_str::<Configs>(&data).expect("Invalid config format")
        } else {
            let mut directory = String::new();
            println!("Enter server directory");
            std::io::stdin()
                .read_line(&mut directory)
                .expect("Failed to read line");
            directory = directory.trim().to_string();
            Configs {
                id: Uuid::new_v4(),
                xms: 1024,
                xmx: 1024,
                dir: directory,
                jar: "server.jar".to_string(),
            }
        }
    }

    pub fn save(&self) {
        let json = serde_json::to_string_pretty(self).expect("Unable to serialize config");
        std::fs::write(Configs::path(), json).expect("Unable to save config");
    }
}
