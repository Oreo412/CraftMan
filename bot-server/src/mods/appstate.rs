use crate::mods::agents::Agent;
use anyhow::Result;
use axum::extract::ws::Message;
use dotenvy;
use protocol::agentactions::AgentActions;
use std::env;
use std::error::Error;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, mpsc};
use twilight_http::Client;

#[derive(Clone)]
pub struct AppState {
    pub connections: Arc<RwLock<HashMap<String, Arc<Agent>>>>,
    pub twilight_client: Arc<Client>,
}

impl AppState {
    pub fn new() -> Self {
        dotenvy::dotenv().ok();
        AppState {
            connections: Arc::new(RwLock::new(HashMap::new())),
            twilight_client: Arc::new(Client::new(
                env::var("DISCORD_TOKEN").expect("Expected a token in the environment"),
            )),
        }
    }

    pub async fn add_connection(&self, user_id: String, Agent: Agent) {
        let mut connections = self.connections.write().await;
        connections.insert(user_id, Arc::new(Agent));
    }

    pub async fn find_connection(&self, id: &str) -> Option<Arc<Agent>> {
        let connections = self.connections.read().await;
        connections.get(id).cloned()
    }

    pub async fn send_message(
        &self,
        id: String,
        message: AgentActions,
    ) -> Result<(), Box<dyn Error>> {
        print!("Sending message to id: {}", id);
        let agent = self
            .find_connection(&id)
            .await
            .ok_or(anyhow::anyhow!("No connection found for id: {}", id))?;
        agent.send_message(message).await?;
        Ok(())
    }
}
