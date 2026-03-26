use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerCommands {
    Say(String),
    Command(String),
    Stop,
}
