#![windows_subsystem = "windows"]

mod app;
mod client;
mod helpers;
mod pages;
mod widget;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use spaces_client::config::ExtendedNetwork;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigBackend {
    Akrond {
        network: ExtendedNetwork,
        prune_point: Option<spaces_protocol::constants::ChainAnchor>,
    },
    Bitcoind {
        network: ExtendedNetwork,
        url: String,
        user: String,
        password: String,
    },
    Spaced {
        network: ExtendedNetwork,
        url: String,
        user: String,
        password: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    path: PathBuf,
    pub backend: Option<ConfigBackend>,
    pub wallet: Option<String>,
}

impl Config {
    fn load(path: PathBuf) -> Self {
        let config: Option<Self> = fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok());
        match config {
            Some(config) => Self { path, ..config },
            None => Self {
                path,
                backend: None,
                wallet: None,
            },
        }
    }

    pub fn save(&self) {
        let config = serde_json::to_string_pretty(&self).unwrap();
        fs::write(&self.path, config).unwrap();
    }

    pub fn remove(&self) {
        fs::remove_file(&self.path).unwrap();
    }

    pub fn reset(&mut self) {
        self.backend = None;
        self.wallet = None;
    }

    pub fn data_dir(&self) -> &std::path::Path {
        self.path.parent().unwrap()
    }
}
pub fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();
    if let Some(service) = akrond::runner::ServiceRunner::parse(&args) {
        if let Err(e) = service.run() {
            eprintln!("{}", e);
            use std::io::Write;
            let _ = std::io::stdout().lock().flush();
            let _ = std::io::stderr().lock().flush();
            std::process::exit(1)
        }
        return Ok(());
    }

    let dirs = ProjectDirs::from("", "", "akron").unwrap();
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir).unwrap();

    let config_path = data_dir.join("config.json");
    let config = Config::load(config_path);
    app::State::run(config)
}
