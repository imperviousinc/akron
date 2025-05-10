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
    Embedded {
        network: ExtendedNetwork,
    },
    Bitcoind {
        network: ExtendedNetwork,
        url: String,
        cookie: String,
        user: String,
        password: String,
    },
    Spaced {
        network: ExtendedNetwork,
        url: String,
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

    pub fn reset(&mut self) {
        self.backend = None;
        self.wallet = None;
    }
}
pub fn main() -> iced::Result {
    let dirs = ProjectDirs::from("", "", "akron").unwrap();
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir).unwrap();

    let config_path = data_dir.join("config.json");
    let config = Config::load(config_path);
    app::State::run(config)
}
