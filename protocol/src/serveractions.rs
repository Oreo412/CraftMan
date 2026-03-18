use crate::query_options::ServerStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerActions {
    PropsUpdate(HashMap<String, String>),
    PropsResponse(Uuid, HashMap<String, String>),
    QueryResponse(Uuid, String, Option<Vec<u8>>, ServerStatus),
    UpdateQuery(u64, u64, ServerStatus),
}

pub enum OneshotResponses {
    PropsResponse(HashMap<String, String>),
    QueryResponse(String, Option<Vec<u8>>, ServerStatus),
}
