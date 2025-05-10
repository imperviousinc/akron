use iced::{
    Element, Task, exit,
    widget::{center, checkbox, column},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use spaces_client::config::ExtendedNetwork;

use crate::{
    branding::*,
    client::Client,
    widget::{
        form::{pick_list, submit_button, text_input, text_label},
        text::error_block,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    error: Option<String>,
    #[serde(skip)]
    path: PathBuf,
    pub spaced_rpc_url: Option<String>,
    pub network: ExtendedNetwork,
    pub wallet: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SpacedRpcUrlToggle(bool),
    SpacedRpcUrlInput(String),
    NetworkSelect(ExtendedNetwork),
    ConnectPress,
    SetError(String),
    SaveAndExit,
}

impl Config {
    pub fn new(path: PathBuf) -> Self {
        Self {
            error: None,
            path,
            spaced_rpc_url: None,
            network: ExtendedNetwork::Mainnet,
            wallet: None,
        }
    }

    pub fn load(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let config = fs::read_to_string(&path)?;
        let config = Self {
            path,
            ..serde_json::from_str(&config)?
        };
        Ok(config)
    }

    pub fn save(&self) {
        let config = serde_json::to_string_pretty(&self).unwrap();
        fs::write(&self.path, config).unwrap();
    }

    pub fn remove(&self) {
        fs::remove_file(&self.path).unwrap();
    }

    pub fn run(self) -> iced::Result {
        iced::application(WINDOW_TITLE, Self::update, Self::view)
            .font(ICONS_FONT.clone())
            .window(iced::window::Settings {
                size: (400.0, 350.0).into(),
                icon: Some(WINDOW_ICON.clone()),
                ..Default::default()
            })
            .theme(|_| BITCOIN_THEME.clone())
            .run_with(move || (self, Task::none()))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        self.error = None;
        match message {
            Message::SpacedRpcUrlToggle(some) => {
                self.spaced_rpc_url = if some {
                    Some("http://127.0.0.1:7225".into())
                } else {
                    None
                };
                Task::none()
            }
            Message::SpacedRpcUrlInput(spaced_rpc_url) => {
                self.spaced_rpc_url = Some(spaced_rpc_url);
                Task::none()
            }
            Message::NetworkSelect(network) => {
                self.network = network;
                Task::none()
            }
            Message::ConnectPress => {
                if let Some(rpc_url) = self.spaced_rpc_url.as_ref() {
                    let network = self.network.to_string();
                    match Client::new(rpc_url) {
                        Ok(client) => Task::future(async move { client.get_server_info().await })
                            .map(move |response| match response {
                                Ok(info) => {
                                    if info.network == network {
                                        Message::SaveAndExit
                                    } else {
                                        Message::SetError("Wrong network".to_string())
                                    }
                                }
                                Err(err) => Message::SetError(err),
                            }),
                        Err(err) => Task::done(Message::SetError(err)),
                    }
                } else {
                    Task::done(Message::SaveAndExit)
                }
            }
            Message::SetError(err) => {
                self.error = Some(err);
                Task::none()
            }
            Message::SaveAndExit => {
                self.save();
                exit()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        center(
            column![
                error_block(self.error.as_ref()),
                column![
                    checkbox("Use standalone spaced node", self.spaced_rpc_url.is_some())
                        .on_toggle(Message::SpacedRpcUrlToggle),
                    text_label("JSON-RPC address"),
                    text_input(
                        "http://127.0.0.1:7225",
                        self.spaced_rpc_url.as_ref().map_or("", |v| v),
                    )
                    .on_input_maybe(
                        self.spaced_rpc_url
                            .as_ref()
                            .map(|_| Message::SpacedRpcUrlInput)
                    ),
                ]
                .spacing(10),
                column![
                    text_label("Chain"),
                    pick_list(
                        [
                            ExtendedNetwork::Mainnet,
                            ExtendedNetwork::Testnet4,
                            ExtendedNetwork::Regtest
                        ],
                        Some(self.network),
                        Message::NetworkSelect
                    )
                ]
                .spacing(10),
                center(submit_button(
                    "Connect",
                    if self.spaced_rpc_url.as_ref().is_some_and(|s| s.is_empty()) {
                        None
                    } else {
                        Some(Message::ConnectPress)
                    }
                ))
            ]
            .spacing(10),
        )
        .padding(20)
        .into()
    }
}
