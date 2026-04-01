use crate::mods::agents::Agent;
use crate::mods::listener;
use anyhow::Result;
use anyhow::anyhow;
use axum::extract::ws::WebSocket;
use dashmap::DashMap;
use futures_util::stream::SplitStream;
use moka::future::Cache;
use nanoid::nanoid;
use protocol::agentactions::AgentActions;
use sqlx::PgPool;
use sqlx::query;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::time::sleep;
use twilight_http::Client;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    connections: Arc<DashMap<Uuid, Arc<Agent>>>,
    uuid_by_guild: Arc<DashMap<u64, Uuid>>,
    pub twilight_client: Arc<Client>,
    pub dbpool: PgPool,
    connection_requests: Arc<Cache<String, PendingRequest>>,
}

impl AppState {
    pub fn new(token: String, dbpool: PgPool) -> Self {
        AppState {
            connections: Arc::new(DashMap::new()),
            uuid_by_guild: Arc::new(DashMap::new()),
            twilight_client: Arc::new(Client::new(token)),
            dbpool,
            connection_requests: Arc::new(
                Cache::builder()
                    .time_to_live(Duration::from_secs(300))
                    .build(),
            ),
        }
    }

    pub async fn create_agent(
        &self,
        id: Uuid,
        receiver: SplitStream<WebSocket>,
        sender: mpsc::UnboundedSender<AgentActions>,
    ) -> Result<()> {
        println!("Creating agent");
        let guild = sqlx::query!("SELECT guild_id FROM servers WHERE agent_id = $1", id)
            .fetch_optional(&self.dbpool)
            .await?;
        if let Some(record) = guild {
            self.build_agent(id, receiver, sender, record.guild_id as u64)
                .await?;
        } else {
            let nanoid = nanoid!(8);
            sender.send(AgentActions::ConnectionKey(nanoid.clone()))?;
            println!("Inserted connection key to cache: {}", &nanoid);
            self.connection_requests
                .insert(nanoid, PendingRequest::new(id, receiver, sender))
                .await;
        }
        Ok(())
    }

    pub async fn build_agent(
        &self,
        id: Uuid,
        receiver: SplitStream<WebSocket>,
        sender: mpsc::UnboundedSender<AgentActions>,
        guild_id: u64,
    ) -> Result<()> {
        self.uuid_by_guild.insert(guild_id, id);
        println!("Should have inserted guild id: {}", guild_id);
        let agent = Arc::new(Agent::new(id, sender, self.dbpool.clone()));
        tokio::spawn(listener::listen(
            receiver,
            agent.clone(),
            self.twilight_client.clone(),
        ));
        self.connections.insert(id, agent);
        Ok(())
    }

    pub async fn verify_agent(&self, code: &str, guild_id: u64) -> Result<()> {
        let (id, receiver, sender) = self
            .connection_requests
            .get(code)
            .await
            .ok_or_else(|| anyhow!("Code not found. Code is either wrong or has expired"))?
            .complete()
            .await;
        query!(
            "INSERT INTO servers (agent_id, guild_id) VALUES ($1, $2)",
            id,
            guild_id as i64
        )
        .execute(&self.dbpool)
        .await?;
        self.build_agent(id, receiver, sender, guild_id).await
    }

    pub fn find_connection(&self, id: &Uuid) -> Result<Arc<Agent>> {
        self.connections
            .get(id)
            .map(|v| v.clone())
            .ok_or_else(|| anyhow!("No connection found for this id"))
    }

    pub fn find_id_by_guild(&self, guild_id: u64) -> Result<Uuid> {
        self.uuid_by_guild
            .get(&guild_id)
            .map(|v| *v)
            .ok_or_else(|| anyhow!("No id found for this guild_id"))
    }

    pub fn find_connection_by_guild(&self, guild_id: u64) -> Result<Arc<Agent>> {
        self.find_connection(self.find_id_by_guild(guild_id)?.as_ref())
    }

    pub async fn send_message(&self, id: Uuid, message: AgentActions) -> Result<()> {
        print!("Sending message to agent connected on guild: {}", id);
        let agent = self.find_connection(&id)?;
        agent.send(message).await?;
        Ok(())
    }

    pub async fn send_by_guild(&self, guild_id: u64, message: AgentActions) -> Result<()> {
        self.send_message(self.find_id_by_guild(guild_id)?, message)
            .await
    }

    pub async fn clean_connections(&self, limit: Duration, cycle_time: Duration) {
        loop {
            println!("Cleaning!");
            let agents: Vec<Arc<Agent>> = self
                .connections
                .iter()
                .map(|entry| entry.value().clone())
                .collect();
            for agent in agents {
                if let Some(since) = agent.since_last_seen().await
                    && since > limit
                {
                    self.connections.remove(&agent.id());
                    println!("Removed: {}", agent.id());
                };
            }
            sleep(cycle_time).await;
        }
    }

    pub fn start_clean_task(&self, limit: Duration, cycle_time: Duration) {
        let run_this = self.clone();
        tokio::spawn(async move { run_this.clean_connections(limit, cycle_time).await });
    }
}

#[derive(Clone)]
pub struct PendingRequest {
    agent_id: Uuid,
    ws_receiver: Arc<Mutex<Option<SplitStream<WebSocket>>>>,
    sender: mpsc::UnboundedSender<AgentActions>,
}

impl PendingRequest {
    pub fn new(
        agent_id: Uuid,
        receiver: SplitStream<WebSocket>,
        sender: mpsc::UnboundedSender<AgentActions>,
    ) -> Self {
        PendingRequest {
            agent_id,
            ws_receiver: Arc::new(Mutex::new(Some(receiver))),
            sender,
        }
    }

    pub async fn complete(
        self,
    ) -> (
        Uuid,
        SplitStream<WebSocket>,
        mpsc::UnboundedSender<AgentActions>,
    ) {
        (
            self.agent_id,
            self.ws_receiver
                .lock()
                .await
                .take()
                .expect("No receiver found"),
            self.sender,
        )
    }
}
