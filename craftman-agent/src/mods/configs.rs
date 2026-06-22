use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{io, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::gui::file_explorer;

#[derive(Serialize, Deserialize, Clone)]
pub struct Configs {
    pub id: Uuid,
    pub xms: u32,
    pub xmx: u32,
    pub dir: String,
    pub jar: String,
    pub run_type: RunType,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum RunType {
    Default,
    Script,
    CustomJar(Vec<String>),
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
        let path = Configs::path();

        if path.exists() {
            let data = std::fs::read_to_string(&path).expect("Failed to read config");

            serde_json::from_str::<Configs>(&data).expect("Invalid config format")
        } else {
            enable_raw_mode().expect("Can not enable raw mode. Fatal error");
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen)
                .expect("Can not execute EnterAlternateScreen. Fatal error");

            let mut terminal = ratatui::init();

            let (file, directory) = file_explorer::blocking_file_selection(&mut terminal).unwrap();
            let run_type = if file.ends_with(".sh") {
                RunType::Script
            } else {
                RunType::Default
            };
            ratatui::restore();
            Configs {
                id: Uuid::new_v4(),
                xms: 1024,
                xmx: 1024,
                dir: directory,
                jar: file,
                run_type,
            }
        }
    }

    pub fn save(&self) {
        let json = serde_json::to_string_pretty(self).expect("Unable to serialize config");
        std::fs::write(Configs::path(), json).expect("Unable to save config");
    }

    pub fn set_xms(mut self, xms: u32) -> Self {
        self.xms = xms;
        self
    }

    pub fn set_xmx(mut self, xmx: u32) -> Self {
        self.xmx = xmx;
        self
    }

    pub fn set_dir(mut self, dir: String) -> Self {
        self.dir = dir;
        self
    }

    pub fn set_jar(mut self, jar: String) -> Self {
        if jar.ends_with(".sh") {
            self.run_type = RunType::Script;
        }
        self.jar = jar;
        self
    }

    pub fn set_run_type(mut self, run_type: RunType) -> Self {
        self.run_type = run_type;
        self
    }
}
