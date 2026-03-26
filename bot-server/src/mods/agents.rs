use anyhow::anyhow;
use axum::extract::ws::Message;
use sqlx::PgPool;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use twilight_http::Client;
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
    query_options::{QueryOptions, ServerStatus},
};
use tokio::{
    sync::{
        RwLock,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    time::interval,
};
use uuid::Uuid;

use crate::mods::bot::chat_channel;

pub struct Agent {
    id: Uuid,
    sender: mpsc::UnboundedSender<AgentActions>,
    pending_requests: Arc<DashMap<Uuid, oneshot::Sender<OneshotResponses>>>,
    chat_channel_cache: RwLock<Cached<Option<Id<ChannelMarker>>>>,
    query_monitor_cache: RwLock<Cached<Option<(Id<MessageMarker>, Id<ChannelMarker>)>>>,
    dbpool: PgPool,
    chat_sender: RwLock<Option<UnboundedSender<String>>>,
}

impl Agent {
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn send(&self, message: AgentActions) -> Result<(), mpsc::error::SendError<AgentActions>> {
        self.sender.send(message)
    }

    pub fn new(id: Uuid, sender: mpsc::UnboundedSender<AgentActions>, dbpool: PgPool) -> Self {
        Agent {
            id,
            sender,
            pending_requests: Arc::new(DashMap::new()),
            chat_channel_cache: RwLock::new(Cached::NotCached),
            query_monitor_cache: RwLock::new(Cached::NotCached),
            dbpool,
            chat_sender: RwLock::new(None),
        }
    }

    pub fn complete_request(&self, id: &Uuid, response: OneshotResponses) -> Result<()> {
        if let Some((_id, sender)) = self.pending_requests.remove(id) {
            sender
                .send(response)
                .map_err(|_| anyhow::anyhow!("receiver dropped"))?;
        } else {
            bail!("No request found");
        }
        Ok(())
    }

    pub async fn request_props(&self) -> Result<HashMap<String, String>> {
        let (sender, receiver) = oneshot::channel::<OneshotResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        let message = AgentActions::RequestProps(request_id);
        self.send(message)?;
        if let Ok(OneshotResponses::PropsResponse(props)) = receiver.await {
            Ok(props)
        } else {
            bail!("Received incorrect response format!");
        }
    }

    pub async fn send_chat(&self, message: String) -> Result<()> {
        if let Some(channel) = self.chat_sender.read().await.clone() {
            channel.send(message)?;
        } else {
            self.send(AgentActions::StopChatStream)?;
            bail!("No open chat connection. Telling agent to stop");
        }
        Ok(())
    }

    pub async fn stop_chat(&self) -> Result<()> {
        if self.chat_sender.write().await.take().is_some() {
            Ok(())
        } else {
            bail!("No chat channel found to stop")
        }
    }

    pub async fn chat_channel(&self) -> Result<Option<Id<ChannelMarker>>> {
        if let Cached::Cached(chat_channel) = *self.chat_channel_cache.read().await {
            Ok(chat_channel)
        } else {
            let chat_channel = sqlx::query!(
                "SELECT chat_channel_id FROM servers WHERE agent_id = $1",
                self.id
            )
            .fetch_one(&self.dbpool)
            .await?
            .chat_channel_id
            .map(|id| Id::new(id as u64));
            *self.chat_channel_cache.write().await = Cached::Cached(chat_channel);
            Ok(chat_channel)
        }
    }

    pub async fn set_chat_channel(&self, chat_channel_id: u64) -> Result<()> {
        sqlx::query!(
            "UPDATE servers SET chat_channel_id = $1 WHERE agent_id = $2",
            chat_channel_id as i64,
            self.id
        )
        .execute(&self.dbpool)
        .await?;
        *self.chat_channel_cache.write().await = Cached::Cached(Some(Id::new(chat_channel_id)));
        Ok(())
    }

    pub async fn edit_props(&self, prop: property) -> Result<HashMap<String, String>> {
        let (sender, receiver) = oneshot::channel::<OneshotResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        self.send(AgentActions::EditProp(request_id, prop));
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
        self.send(AgentActions::StartQuery {
            id: request_id,
            options: QueryOptions::new(options),
            message_id,
            channel_id,
        });
        if let Ok(OneshotResponses::QueryResponse(description, image_bytes, query)) = receiver.await
        {
            Ok((description, image_bytes, query))
        } else {
            bail!("Received incorrect start_query response format!");
        }
    }

    pub async fn start_chat_loop(&self, client: Arc<Client>) -> Result<()> {
        let (chat_sender, chat_receiver) = mpsc::unbounded_channel::<String>();
        tokio::spawn(chat_loop(
            self.chat_channel()
                .await?
                .ok_or_else(|| anyhow!("No chat channel found for this agent"))?,
            chat_receiver,
            client,
        ));
        *self.chat_sender.write().await = Some(chat_sender);

        Ok(())
    }
}

pub async fn chat_loop(
    channel_id: Id<ChannelMarker>,
    mut receiver: UnboundedReceiver<String>,
    client: Arc<Client>,
) -> Result<()> {
    let mut interval = interval(Duration::from_secs(2));
    let mut buffer: Vec<String> = Vec::new();
    loop {
        tokio::select! {
            message = receiver.recv() => {
                match message {
                    Some(text) => {
                        buffer.push(text);
                    }
                    None => break
                }
            }
            _ = interval.tick() => {
                if !buffer.is_empty() {
                    client.create_message(channel_id).content(&buffer.join("\n")).await?;
                    buffer.clear();
                }
            }
        }
    }
    Ok(())
}

enum Cached<T> {
    NotCached,
    Cached(T),
}
