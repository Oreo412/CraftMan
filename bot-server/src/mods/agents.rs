use axum::{extract::ws::Message, http::response};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    sync::Arc,
};

use anyhow::{Result, bail};
use dashmap::DashMap;
use protocol::properties::property;
use protocol::serveractions::OneshotResponses;
use protocol::{
    agentactions::AgentActions,
    query_options::{QueryOptions, QuerySend},
};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;
pub struct Agent {
    pub id: String,
    pub sender: mpsc::UnboundedSender<Message>,
    pub pending_requests: Arc<DashMap<Uuid, oneshot::Sender<OneshotResponses>>>,
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
        let (sender, receiver) = oneshot::channel::<OneshotResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        let message = AgentActions::request_props(request_id);
        self.send_message(message).await?;
        if let Ok(OneshotResponses::PropsResponse(props)) = receiver.await {
            Ok(props)
        } else {
            bail!("Received incorrect response format!");
        }
    }

    pub async fn send_message(&self, message: AgentActions) -> Result<()> {
        self.sender.send(Message::Text(
            serde_json::to_string(&message).unwrap().into(),
        ))?;
        Ok(())
    }

    pub async fn edit_props(&self, prop: property) -> Result<HashMap<String, String>> {
        let (sender, receiver) = oneshot::channel::<OneshotResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        self.send_message(AgentActions::edit_prop(request_id, prop))
            .await?;
        if let Ok(OneshotResponses::PropsResponse(props)) = receiver.await {
            Ok(props)
        } else {
            bail!("Received incorrect response format!");
        }
    }

    pub async fn start_query(
        &self,
        options: HashSet<String>,
        message_id: u64,
        channel_id: u64,
    ) -> Result<(String, Vec<u8>, QuerySend)> {
        let (sender, receiver) = oneshot::channel::<OneshotResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        self.send_message(AgentActions::StartQuery(
            request_id,
            QueryOptions::new(options),
            message_id,
            channel_id,
        ))
        .await?;
        if let Ok(OneshotResponses::QueryResponse(description, image_bytes, query)) = receiver.await
        {
            Ok((description, image_bytes, query))
        } else {
            bail!("Received incorrect start_query response format!");
        }
    }
}
