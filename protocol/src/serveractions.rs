use crate::query_options::ServerStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerActions {
    ConnectAgent(Uuid),
    PropsResponse(Uuid, HashMap<String, String>),
    StartResponse(Uuid),
    StopResponse(Uuid),
    StartChatResponse(Uuid),
    StopChatResponse(Uuid),
    QueryResponse {
        uuid: Uuid,
        description: String,
        image: Option<Vec<u8>>,
        status: ServerStatus,
    },
    UpdateQuery {
        status: ServerStatus,
    },
    UpdateQueryHeader {
        description: String,
        image: Option<Vec<u8>>,
    },
    NewMessage(String),
}

pub enum RequestResponses {
    PropsResponse(HashMap<String, String>),
    QueryResponse(String, Option<Vec<u8>>, ServerStatus),
    StartChatResponse,
    StopChatResponses,
    StartServerResponse,
    StopServerResponse,
}
