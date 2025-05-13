use futures_util::stream::StreamExt;

use std::env;
use std::env::temp_dir;
use std::path::PathBuf;
use tokio::process::{Child, Command};
use std::time::Duration;
use anyhow::Context;
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use tokio::{select};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::time::interval;
use crate::runner::{ServiceCommand, ServiceKind};
use std::process::Stdio;
use reqwest::Client;
use spaces_client::rpc::RootAnchor;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub mod runner;
pub mod services;

#[derive(Debug)]
pub struct Akron {
    stream_tx: mpsc::Sender<AkronCommand>,
    log_tx: Option<broadcast::Sender<String>>,
}

pub struct CheckpointProgress {
    pub downloaded: u64,
    pub total: u64,
}

enum AkronCommand {
    SpawnService {
        kind: ServiceKind,
        args: Vec<String>,
        oneshot: oneshot::Sender<anyhow::Result<()>>,
    },
    Shutdown {
        kind: ServiceKind,
        oneshot: oneshot::Sender<anyhow::Result<()>>,
    },
}

#[allow(dead_code)]
struct Service {
    pub(crate) kind: ServiceKind,
    pub(crate) stream: TcpStream,
    pub(crate) child: Child,
}


impl Akron {
    pub fn create(capture_logs: bool) -> (Self, broadcast::Sender<()>) {
        let (stream_tx, rx) = mpsc::channel::<AkronCommand>(20);
        let shutdown = broadcast::Sender::new(20);
        let log_tx = if capture_logs {
            Some(broadcast::Sender::new(5000))
        } else {
            None
        };

        let task_shutdown = shutdown.clone();
        let err_shutdown = shutdown.clone();
        let task_logs = log_tx.clone();
        std::thread::spawn(move || {
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to start Tokio runtime")
                .block_on(async move {
                    Self::handle_services(rx, task_shutdown, task_logs).await
                });
            if let Err(e) = result {
                error!("Runtime exited with error: {}", e);
                _ = err_shutdown.send(());
            }
        });

        (Self { stream_tx, log_tx }, shutdown)
    }

    pub fn subscribe_logs(&self) -> Option<broadcast::Sender<String>> {
        self.log_tx.clone()
    }

    pub async fn load_checkpoint(
        &self,
        url: &str,
        data_dir: &PathBuf,
        mut progress: Option<mpsc::Sender<CheckpointProgress>>,
    ) -> anyhow::Result<RootAnchor> {
        tokio::fs::create_dir_all(data_dir).await?;
        let spaces_path = data_dir.join("protocol.sdb");
        // Create HTTP client
        let client = Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .context("Failed to send HTTP request")?;

        // Check if request was successful
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP request failed with status: {}", response.status()));
        }

        // Get content length for progress tracking (if available)
        let total = response
            .content_length()
            .context("Failed to get content length, does the path exist?")?;


        let mut file = tokio::fs::File::create(spaces_path.clone())
            .await
            .context("Could not create spaces db file for checkpoint")?;

        // Download and write file in chunks
        let mut downloaded = 0;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk")?;
            file.write_all(&chunk)
                .await
                .context("Failed to write chunk to file")?;
            downloaded += chunk.len() as u64;

            if let Some(progress) = progress.as_mut() {
                _ = progress.send(CheckpointProgress { downloaded, total }).await;
            }
        }

        // Ensure file is fully written
        file.flush().await.context("Failed to flush file")?;
        let root_anchor = tokio::task::spawn_blocking(move || {
            let tmp = temp_dir().join("anchors");
            let db = spaces_client::store::Store::open(spaces_path)?;
            let mut anchors = db.update_anchors(&tmp, 1)?;
            if anchors.is_empty() {
                return Err(anyhow::anyhow!("No Anchors found"));
            }
            _ = std::fs::remove_file(tmp);
            Ok(anchors.remove(0))
        }).await.expect("Could not spawn task")?;
        Ok(root_anchor)
    }

    pub async fn start(&self, kind: ServiceKind, args: Vec<String>) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.stream_tx.send(AkronCommand::SpawnService {
            kind,
            args,
            oneshot: tx,
        }).await.map_err(|e|
            anyhow::anyhow!("Could not spawn service {}: {}", kind.as_str(), e))?;
        rx.await
            .map_err(|e| anyhow::anyhow!("Could not spawn service {}: {}", kind.as_str(), e))?
    }

    pub async fn shutdown(&self, kind: ServiceKind) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.stream_tx.send(AkronCommand::Shutdown {
            kind,
            oneshot: tx,
        }).await.map_err(|e|
            anyhow::anyhow!("Could not shutdown service {}: {}", kind.as_str(), e))?;
        rx.await
            .map_err(|e| anyhow::anyhow!("Could not shutdown service {}: {}", kind.as_str(), e))?
    }

    async fn handle_services(mut rx: mpsc::Receiver<AkronCommand>,
                             shutdown: broadcast::Sender<()>,
                             logs_tx: Option<broadcast::Sender<String>>) -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("Failed to bind TCP listener")?;

        let mut services = Vec::new();
        let mut interval = interval(Duration::from_secs(1));
        let mut shutdown_recv = shutdown.subscribe();

        loop {
            select! {
                Some(cmd) = rx.recv() => {
                   Self::handle_remote_commands(&listener, &mut services, cmd, &logs_tx).await?;
                }
                _ = interval.tick() => {
                    if Self::stopped(&mut services).await {
                        info!("Shutting down one of processes stopped");
                        _ = shutdown.send(());
                        return Ok(());
                    }
                }
                _ = shutdown_recv.recv() => {
                    info!("Received shutdown signal");
                    return Ok(());
                }
            }
        }
    }

    async fn handle_remote_commands(
        listener: &TcpListener,
        services: &mut Vec<Service>,
        cmd: AkronCommand,
        logs_tx: &Option<broadcast::Sender<String>>,
    ) -> anyhow::Result<()> {
        match cmd {
            AkronCommand::SpawnService { kind, args, oneshot } => {
                match Self::handle_start_service(&listener, kind, args, logs_tx.clone()).await {
                    Ok(service) => {
                        // Remove existing ones
                        let pos = services.iter().position(|s| s.kind == service.kind);
                        if let Some(pos) = pos {
                            services.remove(pos).shutdown().await;
                        }
                        services.push(service);
                        _ = oneshot.send(Ok(()));
                    }
                    Err(err) => {
                        _ = oneshot.send(Err(err));
                    }
                }
            }
            AkronCommand::Shutdown { kind, oneshot } => {
                let pos = services.iter().position(|s| s.kind == kind);
                if let Some(pos) = pos {
                    services.remove(pos).shutdown().await;
                }
                _ = oneshot.send(Ok(()));
            }
        }

        Ok(())
    }

    async fn handle_start_service(
        listener: &TcpListener,
        kind: ServiceKind,
        args: Vec<String>,
        log_tx: Option<broadcast::Sender<String>>,
    ) -> anyhow::Result<Service> {
        let addr = listener.local_addr()?.to_string();
        let mut command = Command::new(env::args().next().context("No program name")?);

        #[cfg(unix)]
        command.arg0(format!("akrond-{}", kind.as_str()));

        command
            .arg("--service")
            .arg(kind.as_str())
            .arg("--attach")
            .arg(&addr)
            .args(&args);

        if log_tx.is_some() {
            command.stdin(Stdio::inherit())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
        } else {
            command.stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
        }

        let mut child = command
            .spawn()
            .context(format!("Failed to spawn child service {}", kind.as_str()))?;

        if let Some(log_tx) = log_tx {
            let stdout = child.stdout.take().unwrap();
            let stdout_logs = log_tx.clone();
            tokio::spawn(async move { redirect_logs(stdout_logs, stdout).await });
            let stderr = child.stderr.take().unwrap();
            tokio::spawn(async move { redirect_logs(log_tx, stderr).await });
        }

        // Accept connection from the child
        let (stream, _) = listener
            .accept()
            .await
            .context("Failed to accept child connection")?;

        Ok(Service {
            kind,
            stream,
            child,
        })
    }

    async fn stopped(pipes: &mut Vec<Service>) -> bool {
        for pipe in pipes {
            if !pipe.ping().await {
                return true;
            }
        }
        false
    }
}

impl Service {
    pub async fn ping(&mut self) -> bool {
        self.stream.write(&[ServiceCommand::Ping.to_byte()]).await.is_ok()
    }
    pub async fn shutdown(&mut self) -> bool {
        self.stream.write(&[ServiceCommand::Shutdown.to_byte()]).await.is_ok()
    }
}

async fn redirect_logs<R: tokio::io::AsyncRead + Unpin + Send + 'static>(tx: broadcast::Sender<String>, reader: R) {
    // remove colors
    let r = regex::Regex::new(r"\x1b\[[0-9;]*[mK]").expect("regex");
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let _ = tx.send(r.replace_all(&line, "").into_owned());
    }
}
