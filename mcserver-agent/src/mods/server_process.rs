use protocol::server_commands::ServerCommands;
use protocol::serveractions::ServerActions;
use std::process::Stdio;
use tokio::io::{AsyncWriteExt, Lines};
use tokio::select;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use anyhow::{Result, anyhow};
use tokio::io::AsyncBufReadExt;
use tokio::process::{ChildStdin, ChildStdout};
use tokio::{
    io::BufReader,
    process::{Child, Command},
    sync::{mpsc::UnboundedSender, watch},
};
use tokio_tungstenite::tungstenite::Message;

pub struct ServerProcess {
    child: Child,
    watch_sender: watch::Sender<bool>,
    command_sender: UnboundedSender<ServerCommands>,
    ws_sender: UnboundedSender<ServerActions>,
}

impl ServerProcess {
    pub fn new(
        xms: u32,
        xmx: u32,
        jar: &str,
        dir: &str,
        ws_sender: UnboundedSender<ServerActions>,
    ) -> Result<Self> {
        let mut child = Command::new("java")
            .current_dir(dir)
            .arg(format!("-Xmx{}M", xmx))
            .arg(format!("-Xms{}M", xms))
            .arg("-jar")
            .arg(jar)
            .arg("nogui")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let (watch_sender, watch_receiver) = watch::channel(false);

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("No stdout found for child. Initialization failed"))?;
        let lines = BufReader::new(stdout).lines();
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("No stdin found for child. Initialization failed"))?;

        let (command_sender, command_receiver) = mpsc::unbounded_channel::<ServerCommands>();
        tokio::spawn(commander(command_receiver, stdin));
        tokio::spawn(chat_listener(lines, ws_sender.clone(), watch_receiver));
        Ok(ServerProcess {
            child,
            watch_sender,
            ws_sender,
            command_sender,
        })
    }

    pub fn set_chat(&self, on: bool) -> Result<()> {
        self.watch_sender.send(on)?;
        Ok(())
    }

    pub fn shutdown(self) -> Result<()> {
        self.command_sender.send(ServerCommands::Stop)?;
        Ok(())
    }
}

async fn chat_listener(
    mut lines: Lines<BufReader<ChildStdout>>,
    sender: UnboundedSender<ServerActions>,
    mut watcher: watch::Receiver<bool>,
) -> Result<()> {
    loop {
        select! {
            line = lines.next_line() => {
                match line? {
                    Some(new_message) => {
                        println!("{}", new_message);
                        if *watcher.borrow()
                        {sender.send(ServerActions::NewMessage(new_message))?;
                    }}
                    None => break, // stdout closed
                }
            }

            _ = watcher.changed()=> {
            }
        }
    }

    Ok(())
}

async fn commander(
    mut receiver: UnboundedReceiver<ServerCommands>,
    mut stdin: ChildStdin,
) -> Result<()> {
    while let Some(server_command) = receiver.recv().await {
        match server_command {
            ServerCommands::Say(message) => {
                stdin.write_all(format!("say {}\n", message).as_bytes());
            }
            ServerCommands::Command(command) => {
                stdin.write_all(format!("{}\n", command).as_bytes());
            }
            ServerCommands::Stop => {
                println!("Stopping");
                stdin.write_all(b"stop");
            }
        }
    }
    Ok(())
}
