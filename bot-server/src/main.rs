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

use crate::mods::*;

#[tokio::main]
async fn main() {
    // build our application with a single route

    dotenvy::dotenv().ok();
    let url = env::var("DATABASE_URL").expect("No database url found");
    let token = env::var("DISCORD_TOKEN").expect("No discord token found");
    let app_state = appstate::AppState::new(
        token,
        PgPoolOptions::new()
            .max_connections(10)
            .connect(&url)
            .await
            .expect("Could not connect to database"),
    );
    app_state.start_clean_task(Duration::from_secs(300), Duration::from_secs(120));
    let app = Router::new()
        .route("/", get(|| async { "Axum all over you!" }))
        .route("/ws", get(handler))
        .with_state(app_state.clone());

    tokio::spawn(async move {
        crate::mods::bot::bot_start::start_bot(app_state).await;
    });
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(
    ws: WebSocketUpgrade,
    State(app_state): State<appstate::AppState>,
) -> impl IntoResponse {
    println!("request received for ws");
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
                    println!("Looking for agent connection!");
                    if let Ok(agent) = app_state.find_connection(&id) {
                        println!("Found Agent for this connection. Reconnecting!");
                        agent.reconnect(c_sender).await;
                    } else if let Err(e) = app_state.create_agent(id, receiver, c_sender).await {
                        println!("Error connecting and creating agent: {}", e);
                    }
                    break;
                }
            }
            Message::Close(_) => {
                println!("Client disconnected");
                break;
            }
            _ => {}
        }
    }

    tokio::spawn(write(sender, c_receiver));
}

async fn write(
    mut sender: SplitSink<WebSocket, Message>,
    mut receiver: mpsc::UnboundedReceiver<AgentActions>,
) {
    println!("Starting send loop");
    while let Some(action) = receiver.recv().await {
        println!("message received in channel");
        if sender.send(Message::Text(serde_json::to_string(&action).expect("Serialization of agent action has failed in the write function. This is a major programming error").into())).await.is_err() {
            println!("Client disconnected, stopping send loop");
            break;
        }
    }
}
