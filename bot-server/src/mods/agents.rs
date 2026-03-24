use axum::{extract::ws::Message, http::response};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    sync::Arc,
};
use twilight_model::id::{
    Id,
    marker::{ChannelMarker, MessageMarker},
};

use anyhow::{Result, bail};
use dashmap::DashMap;
use protocol::properties::property;
use protocol::serveractions::OneshotResponses;
use protocol::{
    agentactions::AgentActions,
    query_options::{QueryOptions, QueryStatus, ServerStatus},
};
use tokio::sync::{RwLock, mpsc, oneshot};
use uuid::Uuid;
pub struct Agent {
    pub id: String,
    pub sender: mpsc::UnboundedSender<Message>,
    pub pending_requests: Arc<DashMap<Uuid, oneshot::Sender<OneshotResponses>>>,
    chat_channel_cache: RwLock<Cached<Option<Id<ChannelMarker>>>>,
    query_monitor_cache: RwLock<Cached<Option<(Id<MessageMarker>, Id<ChannelMarker>)>>>,
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
            chat_channel_cache: RwLock::new(Cached::NotCached),
            query_monitor_cache: RwLock::new(Cached::NotCached),
        }
    }

    pub async fn request_props(&self) -> Result<HashMap<String, String>> {
        let (sender, receiver) = oneshot::channel::<OneshotResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        let message = AgentActions::RequestProps(request_id);
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
        self.send_message(AgentActions::EditProp(request_id, prop))
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
    ) -> Result<(String, Option<Vec<u8>>, ServerStatus)> {
        let (sender, receiver) = oneshot::channel::<OneshotResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        self.send_message(AgentActions::StartQuery {
            id: request_id,
            options: QueryOptions::new(options),
            message_id,
            channel_id,
        })
        .await?;
        if let Ok(OneshotResponses::QueryResponse(description, image_bytes, query)) = receiver.await
        {
            Ok((description, image_bytes, query))
        } else {
            bail!("Received incorrect start_query response format!");
        }
    }
}

enum Cached<T> {
    NotCached,
    Cached(T),
}
