use crate::properties::property;
use crate::query_options::QueryOptions;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentActions {
    SvStart,
    SvStop,
    Message(String),
    RequestProps(Uuid),
    EditProp(Uuid, property),
    StartQuery(Uuid, QueryOptions, u64, u64),
}
