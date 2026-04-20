use std::path::PathBuf;

use protocol::agentactions::AgentActions;
use tokio::sync::mpsc::UnboundedSender;
use tui_file_explorer::{FileExplorer, FileExplorerBuilder};

use crate::mods;

pub struct App {
    explorer: FileExplorer,
    state: AppState,
    agent_sender: UnboundedSender<AgentActions>,
    pub config: Config,
}

impl App {
    pub fn new(
        config: &mods::configs::Configs,
        agent_sender: UnboundedSender<AgentActions>,
        directory: String,
        server_file: String,
    ) -> Self {
        let mut explorer = FileExplorer::builder(PathBuf::from(directory))
            .extension_filter(vec!["jar".into()])
            .build();
        if let Some(index) = explorer //I guess this doesn't need to fail if
            //server_file doesn't actually load
            .entries
            .iter()
            .position(|e| e.path == PathBuf::from(&server_file))
        {
            explorer.cursor = index;
        }
        App {
            explorer,
            state: AppState::Default,
            agent_sender,
            config: Config::new(config),
        }
    }
}

pub enum AppState {
    ServerRunning,
    Default,
    Verifying,
}

pub struct Config {
    pub xms: u32,
    pub xmx: u32,
    pub dir: String,
    pub jar: String,
}

impl Config {
    pub fn new(configs: &mods::configs::Configs) -> Self {
        Config {
            xms: configs.xms,
            xmx: configs.xmx,
            dir: configs.dir.clone(),
            jar: configs.jar.clone(),
        }
    }
}
