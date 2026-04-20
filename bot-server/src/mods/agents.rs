use anyhow::anyhow;
use atomic_time::AtomicInstant;
use dashmap::DashMap;
use sqlx::PgPool;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use tokio::sync::oneshot::Sender as OneshotSender;
use tokio::sync::oneshot::channel;
use twilight_http::Client;
use twilight_model::id::{
    Id,
    marker::{ChannelMarker, MessageMarker},
};

use anyhow::{Result, bail};
use protocol::serveractions::RequestResponses;
use protocol::{
    agentactions::AgentActions,
    query_options::{QueryOptions, ServerStatus},
};
use protocol::{properties::property, server_commands::ServerCommands};
use tokio::{
    sync::{
        Mutex, RwLock,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    time::interval,
};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

pub struct Agent {
    id: Uuid,
    sender: Mutex<Option<mpsc::UnboundedSender<AgentActions>>>,
    pending_requests: DashMap<Uuid, OneshotSender<RequestResponses>>,
    chat_channel_cache: RwLock<Cached<Option<Id<ChannelMarker>>>>,
    query_monitor_cache: RwLock<Cached<Option<(Id<ChannelMarker>, Id<MessageMarker>)>>>,
    dbpool: PgPool,
    chat_sender: RwLock<Option<UnboundedSender<String>>>,
    last_seen: Mutex<Option<AtomicInstant>>,
}

impl Agent {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn send(&self, message: AgentActions) -> Result<()> {
        self.sender
            .lock()
            .await
            .as_ref()
            .ok_or_else(|| anyhow!("No connection on this agent"))?
            .send(message)?;
        Ok(())
    }

    pub fn new(id: Uuid, sender: mpsc::UnboundedSender<AgentActions>, dbpool: PgPool) -> Self {
        Agent {
            id,
            sender: Mutex::new(Some(sender)),
            pending_requests: DashMap::new(),
            chat_channel_cache: RwLock::new(Cached::NotCached),
            query_monitor_cache: RwLock::new(Cached::NotCached),
            dbpool,
            chat_sender: RwLock::new(None),
            last_seen: Mutex::new(None),
        }
    }

    #[instrument(skip(self, response))] //response shouldn't be relavent to if anything goes wrong
    //here
    pub async fn complete_request(&self, id: &Uuid, response: RequestResponses) -> Result<()> {
        if let Some((_, sender)) = self.pending_requests.remove(id) {
            debug!("Request sender found. Sending response");
            sender
                .send(response)
                .map_err(|_| anyhow!("Receiver dropped before request could be completed"))?;
        } else {
            bail!("No request found");
        }
        Ok(())
    }

    pub async fn request_props(&self) -> Result<HashMap<String, String>> {
        let (sender, receiver) = channel::<RequestResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        let message = AgentActions::RequestProps(request_id);
        self.send(message).await?;
        if let Ok(RequestResponses::PropsResponse(props)) = receiver.await {
            Ok(props)
        } else {
            bail!("Received incorrect response format, or sender was dropped");
        }
    }

    pub async fn send_chat(&self, message: String) -> Result<()> {
        if let Some(channel) = self.chat_sender.read().await.clone() {
            channel.send(message)?;
        } else {
            bail!("No open chat connection. Telling agent to stop");
        }
        Ok(())
    }

    pub async fn stop_chat_stream(&self) -> Result<()> {
        let (request_sender, request_receiver) = channel::<RequestResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, request_sender);
        if self.chat_sender.write().await.take().is_some() {
            self.send(AgentActions::StopChatStream(request_id)).await?;
            if let Ok(RequestResponses::StopChatResponses) = request_receiver.await {
                Ok(())
            } else {
                bail!("Failed to stop chat stream")
            }
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
        let (sender, receiver) = channel::<RequestResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        self.send(AgentActions::EditProp(request_id, prop)).await?;
        if let Ok(RequestResponses::PropsResponse(props)) = receiver.await {
            Ok(props)
        } else {
            bail!("Received incorrect response format!");
        }
    }

    pub async fn start_query(
        &self,
        options: HashSet<String>,
        message_id: Id<MessageMarker>,
        channel_id: Id<ChannelMarker>,
    ) -> Result<(String, Option<Vec<u8>>, ServerStatus)> {
        sqlx::query!(
            "UPDATE servers SET query_channel_id = $1, query_message_id = $2 WHERE agent_id = $3",
            channel_id.get() as i64,
            message_id.get() as i64,
            self.id
        )
        .execute(&self.dbpool)
        .await?;
        let (sender, receiver) = channel::<RequestResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        self.send(AgentActions::StartQuery(
            request_id,
            QueryOptions::new(options),
        ))
        .await?;
        if let Ok(RequestResponses::QueryResponse(description, image_bytes, query)) = receiver.await
        {
            Ok((description, image_bytes, query))
        } else {
            bail!("Received incorrect start_query response format!");
        }
    }

    pub async fn start_chat_loop(&self, client: Arc<Client>) -> Result<()> {
        let (request_sender, request_receiver) = channel::<RequestResponses>();
        let (chat_sender, chat_receiver) = mpsc::unbounded_channel::<String>();
        tokio::spawn(chat_loop(
            self.chat_channel()
                .await?
                .ok_or_else(|| anyhow!("No chat channel found for this agent"))?,
            chat_receiver,
            client,
        ));
        *self.chat_sender.write().await = Some(chat_sender);
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, request_sender);
        self.send(AgentActions::StartChatStream(request_id)).await?;
        if let Ok(RequestResponses::StartChatResponse) = request_receiver.await {
            Ok(())
        } else {
            bail!("Could not start chat");
        }
    }

    pub async fn start_server(&self) -> Result<()> {
        let (sender, receiver) = channel::<RequestResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        self.send(AgentActions::SvStart(request_id)).await?;
        if let Ok(RequestResponses::StartServerResponse) = receiver.await {
            Ok(())
        } else {
            bail!("Could not start minecraft server");
        }
    }

    pub async fn stop_server(&self) -> Result<()> {
        let (sender, receiver) = channel::<RequestResponses>();
        let request_id = Uuid::new_v4();
        self.pending_requests.insert(request_id, sender);
        self.send(AgentActions::SvStop(request_id)).await?;
        if let Ok(RequestResponses::StopServerResponse) = receiver.await {
            Ok(())
        } else {
            bail!("Could not stop minecraft server");
        }
    }

    pub async fn query_ids(&self) -> Result<Option<(Id<ChannelMarker>, Id<MessageMarker>)>> {
        if let Cached::Cached(ids) = *self.query_monitor_cache.read().await {
            Ok(ids)
        } else {
            let record = sqlx::query!(
                "SELECT query_channel_id, query_message_id FROM servers WHERE agent_id = $1",
                self.id
            )
            .fetch_one(&self.dbpool)
            .await?;
            let ids = if record.query_channel_id.is_some() && record.query_message_id.is_some() {
                Some((
                    Id::new(record.query_channel_id.expect("wtf") as u64),
                    Id::new(record.query_message_id.expect("wtf") as u64),
                ))
            } else {
                None
            };
            *self.query_monitor_cache.write().await = Cached::Cached(ids);
            Ok(ids)
        }
    }

    pub async fn message_chat(&self, command: ServerCommands) -> Result<()> {
        let (request_sender, request_receiver) = channel::<RequestResponses>();
        let uuid = Uuid::new_v4();
        self.pending_requests.insert(uuid, request_sender);
        self.send(AgentActions::ServerCommand(uuid, command))
            .await?;
        if let Ok(RequestResponses::CommandResponse) = request_receiver.await {
            Ok(())
        } else {
            bail!("Could not send command to server")
        }
    }

    #[instrument(skip(self))]
    pub async fn lost_connection(&self) {
        *self.last_seen.lock().await = Some(AtomicInstant::now());
        *self.sender.lock().await = None;
        info!("Lost Connection, last seen and sender changed!");
    }

    pub async fn reconnect(&self, sender: mpsc::UnboundedSender<AgentActions>) {
        *self.sender.lock().await = Some(sender);
        *self.last_seen.lock().await = None;
    }

    pub async fn since_last_seen(&self) -> Option<Duration> {
        self.last_seen
            .lock()
            .await
            .as_ref()
            .map(|time| time.load(std::sync::atomic::Ordering::Relaxed).elapsed())
    }
}

#[instrument(skip(client, receiver))]
pub async fn chat_loop(
    channel_id: Id<ChannelMarker>,
    mut receiver: UnboundedReceiver<String>,
    client: Arc<Client>,
) -> Result<()> {
    let mut interval = interval(Duration::from_secs(2));
    let mut buffer: Vec<String> = Vec::new();
    info!("Starting chat loop");
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
    info!("Closing chat loop");
    Ok(())
}

enum Cached<T> {
    NotCached,
    Cached(T),
}
