use anyhow::Result;
use axum::extract::ws::Message;
use protocol::agentactions::AgentActions;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, mpsc};

#[derive(Clone)]
pub struct AppState {
    pub connections: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>,
}

impl AppState {
    // You can add methods for managing the app state here
    pub fn new() -> Self {
        AppState {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, user_id: String, sender: mpsc::UnboundedSender<Message>) {
        let mut connections = self.connections.write().await;
        connections.insert(user_id, sender);
    }

    pub async fn find_connection(&self, id: &str) -> Option<mpsc::UnboundedSender<Message>> {
        let connections = self.connections.read().await;
        connections.get(id).cloned()
    }

    pub async fn send_message(&self, id: String, message: AgentActions) -> Result<()> {
        print!("Sending message to id: {}", id);
        let sender = self
            .find_connection(&id)
            .await
            .ok_or(anyhow::anyhow!("No connection found for id: {}", id))?;
        sender.send(Message::Text(
            serde_json::to_string(&message).unwrap().into(),
        ))?;
        Ok(())
    }
}
