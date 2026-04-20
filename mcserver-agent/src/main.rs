mod gui;
mod mods;
use crate::mods::{server_handler::ServerHandler, *};
use connect::connect;
use std::time::Duration;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = configs::Configs::new();

    let mut handler = ServerHandler::new(config);

    loop {
        match connect(&mut handler).await {
            Ok(()) => {
                println!("Disconnected. Reconnecting...");
            }
            Err(e) => {
                eprintln!("Connection failed: {e}");
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
