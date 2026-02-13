use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum property {
    allow_flight,
}
