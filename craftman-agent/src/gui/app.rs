use std::collections::VecDeque;

use crate::mods::configs::Configs;
use anyhow::{Result, bail};
use ratatui_explorer::FileExplorer;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use crate::gui::gui_actions::{ConfigRequest, EditRequestReturn};

pub struct App {
    pub state: AppState,
    agent_sender: UnboundedSender<ConfigRequest>,
    pub server_running: bool,
    pub stdout: VecDeque<String>,
    pub config: Configs,
}

impl App {
    pub fn new(config: Configs, agent_sender: UnboundedSender<ConfigRequest>) -> Self {
        App {
            state: AppState::Default,
            agent_sender,
            server_running: false,
            stdout: VecDeque::new(),
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

    pub async fn update_config(&mut self) -> Result<()> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<Configs>();
        tracing::info!("Updating tui app config");
        self.agent_sender
            .send(ConfigRequest::Request(oneshot_sender))?;
        tracing::info!("Update config request sent, awaiting updated config");
        self.config = oneshot_receiver.await?;
        tracing::info!("Updated config received, tui app updated!");
        Ok(())
    }

    pub async fn edit_config(&mut self, config: Configs) -> Result<()> {
        tracing::info!("Beginning config edit");
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<EditRequestReturn>();
        self.agent_sender
            .send(ConfigRequest::Edit(oneshot_sender, config))?;
        tracing::info!("New config sent, awaiting return");
        if let EditRequestReturn::EditInvalid(err) = oneshot_receiver.await? {
            bail!("Could not edit config: {}", err);
        }
        tracing::info!("Edited config. Updating app with new config");
        self.update_config().await
    }
}

pub enum AppState {
    Default,
    FileSelection(FileExplorer),
    Validate(String),
    EditMemory(EditMemory),
    Exiting,
}

pub struct EditMemory {
    pub xms_string: String,
    pub xms: Option<u32>,
    pub xmx_string: String,
    pub xmx: Option<u32>,
    pub state: EditMemoryState,
    pub invalid_input: bool,
}

impl EditMemory {
    pub fn new() -> Self {
        EditMemory {
            xms_string: String::new(),
            xms: None,
            xmx_string: String::new(),
            xmx: None,
            state: EditMemoryState::Editxms,
            invalid_input: false,
        }
    }

    pub fn verify(&mut self) -> Result<()> {
        self.xms = if self.xms_string.ends_with('G') {
            let mut xms = self.xms_string.clone();
            xms.pop();
            Some(xms.parse::<u32>()? * 1024)
        } else {
            Some(self.xms_string.parse::<u32>()?)
        };

        self.xmx = if self.xmx_string.ends_with('G') {
            let mut xmx = self.xmx_string.clone();
            xmx.pop();
            Some(xmx.parse::<u32>()? * 1024)
        } else {
            Some(self.xmx_string.parse::<u32>()?)
        };
        Ok(())
    }
}

#[derive(PartialEq, Eq)]
pub enum EditMemoryState {
    Editxms,
    Editxmx,
    IsThisCorrect,
}
