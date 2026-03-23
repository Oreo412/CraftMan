use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SendChat {
    NewMessage(u64, String),
}
