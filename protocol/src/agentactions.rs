use crate::query_options::QueryOptions;
use crate::{properties::Property, server_commands::ServerCommands};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentActions {
    SvStart(Uuid),
    SvStop(Uuid),
    Message(String),
    RequestProps(Uuid),
    EditProp(Uuid, Property),
    StartQuery(Uuid, QueryOptions),
    StopQuery,
    StartChatStream(Uuid),
    StopChatStream(Uuid),
    ValidationToken(String),
    Validate,
    ServerCommand(Uuid, ServerCommands),
}
