use std::collections::HashMap;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerActions {
    props_update(HashMap<String, String>),
    response_props(Uuid, HashMap<String, String>),
}
