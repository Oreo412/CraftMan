use crate::properties::property;
use crate::query_options::QueryOptions;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentActions {
    sv_start,
    sv_stop,
    message(String),
    request_props(Uuid),
    edit_prop(Uuid, property),
    StartQuery(Uuid, QueryOptions, u64, u64),
}
