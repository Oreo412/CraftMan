mod mods;
use axum::{
    Router,
    extract::State,
    extract::ws::*,
    response::{IntoResponse, Response},
    routing::{any, get},
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use std::fmt::Display;
use std::time::Duration;
use tokio::{sync::mpsc, time::*};

use crate::mods::*;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app_state = appstate::AppState::new();
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

async fn handle_socket(mut socket: WebSocket, app_state: appstate::AppState) {
    let (mut sender, mut receiver) = socket.split();

    let (mut c_sender, mut c_receiver) = mpsc::unbounded_channel::<Message>();

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                app_state
                    .add_connection(
                        text.to_string(),
                        agents::Agent::new(text.to_string(), c_sender),
                    )
                    .await;
                println!("Registered new connection with id: {}", text);
                tokio::spawn(listener::listen(
                    receiver,
                    app_state.find_connection(&text).await.unwrap(),
                    app_state.twilight_client.clone(),
                ));
                break;
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

async fn read(mut receiver: SplitStream<WebSocket>) {
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                println!("Client says: {}", text);
            }
            Message::Close(_) => {
                println!("Client disconnected");
                break;
            }
            _ => {}
        }
    }
}

async fn write(
    mut sender: SplitSink<WebSocket, Message>,
    mut receiver: mpsc::UnboundedReceiver<Message>,
) {
    println!("Starting send loop");
    while let Some(msg) = receiver.recv().await {
        println!("message received in channel");
        if sender.send(msg).await.is_err() {
            println!("Client disconnected, stopping send loop");
            break;
        }
    }
}
