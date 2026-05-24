use std::{collections::VecDeque, path::PathBuf};

use mods::configs::Configs;
use protocol::agentactions::AgentActions;
use tokio::sync::mpsc::UnboundedSender;
use tui_file_explorer::{FileExplorer, FileExplorerBuilder};

use crate::{gui::gui_actions::ConfigRequest, mods};

pub struct App {
    explorer: FileExplorer,
    state: AppState,
    agent_sender: UnboundedSender<ConfigRequest>,
    pub server_running: bool,
    pub stdout: VecDeque<String>,
    pub scroll: u16,
    pub config: Configs,
}

impl App {
    pub fn new(config: Configs, agent_sender: UnboundedSender<ConfigRequest>) -> Self {
        let mut explorer = FileExplorer::builder(PathBuf::from(&config.dir))
            .extension_filter(vec!["jar".into()])
            .build();
        if let Some(index) = explorer //I guess this doesn't need to fail if
            //server_file doesn't actually load
            .entries
            .iter()
            .position(|e| e.path == PathBuf::from(&config.jar))
        {
            explorer.cursor = index;
        }
        App {
            explorer,
            state: AppState::Default,
            agent_sender,
            server_running: false,
            stdout: VecDeque::new(),
            scroll: 0,
            config,
        }
    }

    pub fn start_validation(&mut self, key: String) {
        self.state = AppState::Validate(key);
    }

    pub fn complete_validation(&mut self) {
        if matches!(self.state, AppState::Validate(_)) {
            self.state = AppState::Default;
        }
    }
}

enum AppState {
    Default,
    Validate(String),
}
