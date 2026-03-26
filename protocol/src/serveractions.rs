use crate::query_options::ServerStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerActions {
    ConnectAgent(Uuid),
    PropsUpdate(HashMap<String, String>),
    PropsResponse(Uuid, HashMap<String, String>),
    StartResponse(Uuid, bool),
    StopResponse(Uuid, bool),
    QueryResponse {
        uuid: Uuid,
        description: String,
        image: Option<Vec<u8>>,
        status: ServerStatus,
    },
    UpdateQuery {
        channel_id: u64,
        message_id: u64,
        status: ServerStatus,
    },
    UpdateQueryHeader {
        channel_id: u64,
        message_id: u64,
        description: String,
        image: Option<Vec<u8>>,
    },
    NewMessage(String),
}

pub enum OneshotResponses {
    PropsResponse(HashMap<String, String>),
    QueryResponse(String, Option<Vec<u8>>, ServerStatus),
}
