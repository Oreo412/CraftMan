use crate::query_options::QuerySend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerActions {
    props_update(HashMap<String, String>),
    PropsResponse(Uuid, HashMap<String, String>),
    QueryResponse(Uuid, String, Vec<u8>, QuerySend),
}

pub enum OneshotResponses {
    PropsResponse(HashMap<String, String>),
    QueryResponse(String, Vec<u8>, QuerySend),
}
