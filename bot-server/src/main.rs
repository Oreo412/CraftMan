mod mods;
use axum::{Router, extract::State, extract::ws::*, response::IntoResponse, routing::get};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
};
use protocol::{agentactions::AgentActions, serveractions::ServerActions};
use sqlx::postgres::PgPoolOptions;
use std::{env, time::Duration};
use tokio::sync::mpsc;
use tracing::{Instrument, debug, error, info, info_span, instrument};

use crate::mods::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();
    let url = env::var("DATABASE_URL").expect("No database url found");
    let token = env::var("DISCORD_TOKEN").expect("No discord token found");
    info!("Evironment variables initialized");
    let app_state = appstate::AppState::new(
        token,
        PgPoolOptions::new()
            .max_connections(10)
            .connect(&url)
            .await
            .expect("Could not connect to database"),
    );
    info!("Appstate created");
    app_state.start_clean_task(Duration::from_secs(300), Duration::from_secs(120));
    info!("Cleaning task started");
    let app = Router::new()
        .route("/", get(|| async { "Axum all over you!" }))
        .route("/ws", get(handler))
        .with_state(app_state.clone());
    info!("App created");

    tokio::spawn(async move {
        crate::mods::bot::bot_start::start_bot(app_state).await;
    });
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    info!("Server started")
}

#[instrument(skip(app_state, ws))]
async fn handler(
    ws: WebSocketUpgrade,
    State(app_state): State<appstate::AppState>,
) -> impl IntoResponse {
    info!("New websocket connection request received");
    ws.on_upgrade(move |socket| handle_socket(socket, app_state))
}

async fn handle_socket(socket: WebSocket, app_state: appstate::AppState) {
    let (sender, mut receiver) = socket.split();

    let (c_sender, c_receiver) = mpsc::unbounded_channel::<AgentActions>();

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let ServerActions::ConnectAgent(id) =
                    serde_json::from_str::<ServerActions>(text.as_str()).expect("Uh oh")
                {
                    let connection_span = info_span!("Connection", %id);
                    let _entered = connection_span.enter();

                    if let Ok(agent) = app_state.find_connection(&id) {
                        debug!("Found Agent for this connection. Reconnecting!");
                        agent.reconnect(c_sender).await;
                    } else if let Err(e) = app_state.create_agent(id, receiver, c_sender).await {
                        error!("Error connecting and creating agent: {}", e);
                    }
                    tokio::spawn(write(sender, c_receiver).instrument(connection_span.clone()));
                    break;
                }
            }
            Message::Close(_) => {
                info!("Client disconnected");
                break;
            }
            _ => {}
        }
    }
}

#[instrument(skip(sender, receiver))]
async fn write(
    mut sender: SplitSink<WebSocket, Message>,
    mut receiver: mpsc::UnboundedReceiver<AgentActions>,
) {
    debug!("Starting send loop");
    while let Some(action) = receiver.recv().await {
        debug!("message received in channel");
        if sender.send(Message::Text(serde_json::to_string(&action).expect("Serialization of agent action has failed in the write function. This is a major programming error").into())).await.is_err() {
            info!("Client disconnected, stopping send loop");
            break;
        }
    }
    info!("MPSC receiver for write loop returned none. Stopping loop");
}
