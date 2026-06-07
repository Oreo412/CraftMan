use crate::gui::gui_actions::ConfigRequest;
use crate::gui::tui::GuiEvents;
use crate::mods::{server_handler::ServerHandler, *};
use futures_util::stream::StreamExt;
use protocol::serveractions::ServerActions;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::connect_async;

const URL: &str = match option_env!("URL") {
    Some(url) => url,
    None => "ws://localhost:3000/craftman",
};

pub async fn connect(
    handler: &mut ServerHandler,
    agent_from_tui: &mut UnboundedReceiver<ConfigRequest>,
    agent_to_tui: UnboundedSender<GuiEvents>,
    sender: UnboundedSender<ServerActions>,
    receiver: &mut UnboundedReceiver<ServerActions>,
) -> anyhow::Result<()> {
    tracing::info!("Trying to connect to: {}", URL);
    let (ws_stream, _) = connect_async(URL).await?;

    let (ws_write, ws_read) = ws_stream.split();

    sender.send(ServerActions::ConnectAgent(handler.id()))?;
    tracing::info!("Connected to server!");

    listener::listen(
        ws_read,
        ws_write,
        sender,
        handler,
        agent_from_tui,
        agent_to_tui.clone(),
        receiver,
    )
    .await?;

    Ok(())
}
