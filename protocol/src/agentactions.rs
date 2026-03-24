use crate::properties::property;
use crate::query_options::QueryOptions;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentActions {
    SvStart(Uuid),
    SvStop(Uuid),
    Message(String),
    RequestProps(Uuid),
    EditProp(Uuid, property),
    StartQuery {
        id: Uuid,
        options: QueryOptions,
        message_id: u64,
        channel_id: u64,
    },
    SetChatChannel(u64),
}
