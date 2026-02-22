use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryOptions {
    version: bool,
    player_count: bool,
    player_list: bool,
    description: bool,
    map: bool,
    gamemode: bool,
    software: bool,
    plugins: bool,
    mods: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        QueryOptions {
            version: false,
            player_count: false,
            player_list: false,
            description: false,
            map: false,
            gamemode: false,
            software: false,
            plugins: false,
            mods: false,
        }
    }
}

impl QueryOptions {
    pub fn new(options: HashSet<String>) -> Self {
        QueryOptions {
            version: options.contains("version"),
            player_count: options.contains("player count"),
            player_list: options.contains("player list"),
            description: options.contains("description"),
            map: options.contains("map"),
            gamemode: options.contains("gamemode"),
            software: options.contains("software"),
            plugins: options.contains("plugins"),
            mods: options.contains("mods"),
        }
    }

    pub fn version(&mut self) -> bool {
        self.version
    }

    pub fn player_count(&mut self) -> bool {
        self.player_count
    }

    pub fn player_list(&mut self) -> bool {
        self.player_list
    }

    pub fn description(&mut self) -> bool {
        self.description
    }

    pub fn map(&mut self) -> bool {
        self.map
    }

    pub fn gamemode(&mut self) -> bool {
        self.gamemode
    }

    pub fn software(&mut self) -> bool {
        self.software
    }

    pub fn plugins(&mut self) -> bool {
        self.plugins
    }

    pub fn mods(&mut self) -> bool {
        self.mods
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuerySend {
    version: Option<String>,
    player_count: Option<String>,
    player_list: Option<Vec<String>>,
    description: Option<String>,
    map: Option<String>,
    gamemode: Option<String>,
    software: Option<String>,
    plugins: Option<Vec<String>>,
    mods: Option<Vec<String>>,
}

impl Default for QuerySend {
    fn default() -> Self {
        QuerySend {
            version: None,
            player_count: None,
            player_list: None,
            description: None,
            map: None,
            gamemode: None,
            software: None,
            plugins: None,
            mods: None,
        }
    }
}

impl QuerySend {
    pub fn set_version(&mut self, version: String) {
        self.version = Some(version);
    }

    pub fn set_player_count(&mut self, player_count: String) {
        self.player_count = Some(player_count);
    }

    pub fn set_player_list(&mut self, player_list: Vec<String>) {
        self.player_list = Some(player_list);
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn set_map(&mut self, map: Option<String>) {
        self.map = map;
    }

    pub fn set_gamemode(&mut self, gamemode: Option<String>) {
        self.gamemode = gamemode;
    }

    pub fn set_software(&mut self, software: Option<String>) {
        self.software = software;
    }

    pub fn set_plugins(&mut self, plugins: Option<Vec<String>>) {
        self.plugins = plugins;
    }

    pub fn set_mods(&mut self, mods: Option<Vec<String>>) {
        self.mods = mods;
    }

    pub fn version(&self) -> Option<&String> {
        self.version.as_ref()
    }

    pub fn player_count(&self) -> Option<&String> {
        self.player_count.as_ref()
    }

    pub fn player_list(&self) -> Option<&Vec<String>> {
        self.player_list.as_ref()
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn map(&self) -> Option<&String> {
        self.map.as_ref()
    }

    pub fn gamemode(&self) -> Option<&String> {
        self.gamemode.as_ref()
    }

    pub fn software(&self) -> Option<&String> {
        self.software.as_ref()
    }

    pub fn plugins(&self) -> Option<&Vec<String>> {
        self.plugins.as_ref()
    }

    pub fn mods(&self) -> Option<&Vec<String>> {
        self.mods.as_ref()
    }
}
