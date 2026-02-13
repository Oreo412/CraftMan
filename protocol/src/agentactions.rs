use crate::properties::property;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentActions {
    sv_start,
    sv_stop,
    message(String),
    request_props(Uuid),
    edit_prop(Uuid, property),
}
