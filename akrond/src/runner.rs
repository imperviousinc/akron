use crate::services::spaces;
use crate::services::yuki;
use anyhow::Context;
use log::info;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::broadcast;

pub struct ServiceRunner {
    attach_addr: Option<String>,
    kind: ServiceKind,
    args: Vec<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ServiceKind {
    Spaces,
    Yuki,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceCommand {
    Ping,
    Shutdown,
}

impl ServiceRunner {
    pub fn parse(args: &[String]) -> Option<Self> {
        let mut args = args.iter().cloned().collect();
        let service = read_arg("--service", &mut args)?;
        let kind = ServiceKind::from_str(&service)?;
        let attach_addr = read_arg("--attach", &mut args);

        Some(Self {
            attach_addr,
            kind,
            args,
        })
    }

    pub fn run(self) -> anyhow::Result<()> {
        let rt = tokio::runtime::Runtime::new().expect("Failed to build tokio runtime");

        rt.block_on(async {
            let sigterm = tokio::signal::ctrl_c();
            let shutdown = broadcast::Sender::new(1);
            if let Some(addr) = self.attach_addr {
                ServiceRunner::attach(self.kind, addr, shutdown.clone()).await?;
            }

            let sigterm_shutdown = shutdown.clone();
            tokio::spawn(async move {
                sigterm.await.expect("could not listen for shutdown");
                let _ = sigterm_shutdown.send(());
            });

            match self.kind {
                ServiceKind::Spaces => spaces::main(self.args, shutdown).await?,
                ServiceKind::Yuki => yuki::main(self.args, shutdown).await?,
            }
            Ok(())
        })
    }

    pub async fn attach(
        kind: ServiceKind,
        addr: String,
        shutdown: broadcast::Sender<()>,
    ) -> anyhow::Result<()> {
        let mut stream = TcpStream::connect(&addr)
            .await
            .context("Failed to connect to parent")?;
        // Spawn the reader/shutdown watcher
        tokio::task::spawn(async move {
            let mut shutdown_rx = shutdown.subscribe();
            let mut buf = [0u8; 1];

            loop {
                select! {
                  _ = shutdown_rx.recv() => {
                      info!("{}: shutdown requested, exiting …", kind.as_str());
                      return;
                  }
                  result = stream.read(&mut buf) => match result {
                      Ok(0) => {
                          info!("{}: parent disconnected, exiting …", kind.as_str());
                          let _ = shutdown.send(());
                          return;
                      }
                      Err(e) => {
                          info!("{}: read error {:?}, exiting …", kind.as_str(), e);
                          let _ = shutdown.send(());
                          return;
                      }
                      Ok(_) => {
                          match ServiceCommand::from_byte(buf[0]) {
                              Some(ServiceCommand::Ping) => {
                                  // do your ping logic
                                  tokio::time::sleep(Duration::from_millis(20)).await;
                              }
                              Some(ServiceCommand::Shutdown) => {
                                  info!("{}: shutdown command requested", kind.as_str());
                                  let _ = shutdown.send(());
                                  return;
                              }
                              None => {
                                  info!("{}: unknown command {}, ignoring", kind.as_str(), buf[0]);
                              }
                          }
                      }
                  }
                }
            }
        });

        Ok(())
    }
}

impl ServiceKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            ServiceKind::Spaces => "spaces",
            ServiceKind::Yuki => "yuki",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "spaces" => Some(ServiceKind::Spaces),
            "yuki" => Some(ServiceKind::Yuki),
            _ => None,
        }
    }
}

impl ServiceCommand {
    pub(crate) fn to_byte(&self) -> u8 {
        match self {
            ServiceCommand::Ping => 0,
            ServiceCommand::Shutdown => 1,
        }
    }

    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(ServiceCommand::Ping),
            1 => Some(ServiceCommand::Shutdown),
            _ => None,
        }
    }
}

fn read_arg(arg: &str, args: &mut Vec<String>) -> Option<String> {
    if let Some(pos) = args.iter().position(|a| a == arg) {
        args.remove(pos);
        if pos < args.len() {
            return Some(args.remove(pos));
        }
    }
    None
}
