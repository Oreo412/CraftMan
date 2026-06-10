use protocol::server_commands::ServerCommands;
use protocol::serveractions::ServerActions;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncWriteExt, Lines};
use tokio::select;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use anyhow::{Result, anyhow};
use tokio::io::AsyncBufReadExt;
use tokio::process::{ChildStderr, ChildStdin, ChildStdout};
use tokio::{
    io::BufReader,
    process::Command,
    sync::{mpsc::UnboundedSender, watch},
};

use crate::mods::configs::RunType;

pub struct ServerProcess {
    watch_sender: watch::Sender<bool>,
    command_sender: UnboundedSender<ServerCommands>,
}

impl ServerProcess {
    pub fn new(
        xms: u32,
        xmx: u32,
        jar: &str,
        dir: &str,
        ws_sender: UnboundedSender<ServerActions>,
        run_type: &RunType,
    ) -> Result<Self> {
        let mut child = match run_type {
            RunType::Default => Command::new("java")
                .current_dir(dir)
                .arg(format!("-Xmx{}M", xmx))
                .arg(format!("-Xms{}M", xms))
                .arg("-jar")
                .arg(jar)
                .arg("nogui")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?,
            RunType::Script => {
                update_user_jvm_args(&Path::new(dir).join("user_jvm_args.txt"), xms, xmx)?;
                Command::new(format!("./{}", jar))
                    .current_dir(dir)
                    .arg("nogui")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?
            }
            RunType::CustomJar(args) => {
                let mut process = Command::new("java");
                process.current_dir(dir);

                if args.iter().any(|arg| arg == "@user_jvm_args.txt") {
                    update_user_jvm_args(&Path::new(dir).join("user_jvm_args.txt"), xms, xmx)?;
                } else if !args
                    .iter()
                    .any(|arg| arg.starts_with("-Xms") || arg.starts_with("-Xmx"))
                {
                    process
                        .arg(format!("-Xms{}M", xms))
                        .arg(format!("-Xmx{}M", xmx));
                }

                for arg in args.iter() {
                    process.arg(arg);
                }

                process
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?
            }
        };
        let (watch_sender, watch_receiver) = watch::channel(false);

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("No stdout found for child. Initialization failed"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("No stderr found for child. Initialization failed"))?;
        let lines = BufReader::new(stdout).lines();
        let err_lines = BufReader::new(stderr).lines();
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("No stdin found for child. Initialization failed"))?;

        let (command_sender, command_receiver) = mpsc::unbounded_channel::<ServerCommands>();
        tokio::spawn(commander(command_receiver, stdin));
        tokio::spawn(chat_listener(
            lines,
            err_lines,
            ws_sender.clone(),
            watch_receiver,
        ));
        Ok(ServerProcess {
            watch_sender,
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

    pub fn send_command(&self, command: ServerCommands) -> Result<()> {
        self.command_sender.send(command)?;
        Ok(())
    }
}

async fn chat_listener(
    mut lines: Lines<BufReader<ChildStdout>>,
    mut err_lines: Lines<BufReader<ChildStderr>>,
    sender: UnboundedSender<ServerActions>,
    mut watcher: watch::Receiver<bool>,
) -> Result<()> {
    loop {
        select! {
            line = lines.next_line() => {
                match line? {
                    Some(new_message) => {
                        tracing::info!("{}", new_message);
                        if *watcher.borrow()
                        {sender.send(ServerActions::ChatMessage(new_message))?;
                    }}
                    None => break, // stdout closed
                }
            }

            line = err_lines.next_line() => {
                match line? {
                    Some(new_message) => {
                        tracing::info!("{}", new_message);
                        if *watcher.borrow()
                        {sender.send(ServerActions::ChatMessage(new_message))?;
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
                stdin
                    .write_all(format!("say {}\n", message).as_bytes())
                    .await?;
            }
            ServerCommands::Command(command) => {
                stdin.write_all(format!("{}\n", command).as_bytes()).await?;
            }
            ServerCommands::Stop => {
                tracing::info!("Stopping");
                stdin.write_all(b"stop").await?;
            }
        }
    }
    Ok(())
}

fn update_user_jvm_args(path: &Path, xms_mb: u32, xmx_mb: u32) -> std::io::Result<()> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();

    let mut lines: Vec<String> = existing
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("-Xms") && !trimmed.starts_with("-Xmx")
        })
        .map(|line| line.to_string())
        .collect();

    lines.insert(0, format!("-Xmx{}M", xmx_mb));
    lines.insert(0, format!("-Xms{}M", xms_mb));

    let output = format!("{}\n", lines.join("\n"));
    std::fs::write(path, output)
}
