use axum::{extract::ws::Message, http::response};
use std::{collections::HashMap, error::Error, sync::Arc};

use anyhow::Result;
use dashmap::DashMap;
use protocol::agentactions::AgentActions;
use protocol::properties::property;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;
pub struct Agent {
    pub id: String,
    pub sender: mpsc::UnboundedSender<Message>,
    pub pending_requests: Arc<DashMap<Uuid, oneshot::Sender<HashMap<String, String>>>>,
}

impl Agent {
    pub fn send(&self, message: Message) -> Result<(), mpsc::error::SendError<Message>> {
        self.sender.send(message)
    }

    pub fn new(id: String, sender: mpsc::UnboundedSender<Message>) -> Self {
        Agent {
            id,
            sender,
            pending_requests: Arc::new(DashMap::new()),
        }
    }

    pub async fn request_props(&self) -> Result<HashMap<String, String>> {
        let (sender, receiver) = oneshot::channel::<HashMap<String, String>>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        let message = AgentActions::request_props(request_id);
        self.send_message(message).await?;
        Ok(receiver.await?)
    }

    pub async fn send_message(&self, message: AgentActions) -> Result<()> {
        self.sender.send(Message::Text(
            serde_json::to_string(&message).unwrap().into(),
        ))?;
        Ok(())
    }

    pub async fn edit_props(&self, prop: property) -> Result<HashMap<String, String>> {
        let (sender, receiver) = oneshot::channel::<HashMap<String, String>>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        self.send_message(AgentActions::edit_prop(request_id, prop))
            .await?;
        Ok(receiver.await?)
    }
}
