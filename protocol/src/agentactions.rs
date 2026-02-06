use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentActions {
    sv_start,
    sv_stop,
    message(String),
}
