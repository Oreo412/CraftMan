use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentActions {
    sv_start,
    message(String),
}
