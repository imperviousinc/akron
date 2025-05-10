use iced::{
    Center, Element, Task,
    widget::{button, column, container, horizontal_space, row},
};

use spaces_client::config::ExtendedNetwork;

use crate::{
    Config, ConfigBackend,
    client::{Client, ClientResult, ServerInfo},
    widget::{
        form::{Form, submit_button},
        icon::{Icon, button_icon, text_icon},
        text::{error_block, text_big, text_bold},
    },
};

#[derive(Debug)]
pub struct State {
    config: Config,
    client: Option<Client>,
    connected: bool,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackendSet(ConfigBackend),
    NetworkSelect(ExtendedNetwork),
    UrlInput(String),
    CookieInput(String),
    UserInput(String),
    PasswordInput(String),
    Connect,
    ConnectResult(ClientResult<ServerInfo>),
    ListWalletsResult(ClientResult<Vec<String>>),
    Reset,
    Disconnect,
    CreateWallet,
    ImportWallet,
    ImportWalletPicked(Result<String, String>),
    SetWalletResult(Result<String, String>),
}

pub enum Action {
    Return(Config, Client),
    Task(Task<Message>),
}

impl Action {
    fn none() -> Action {
        Action::Task(Task::none())
    }
}

impl State {
    pub fn run(config: Config) -> (Self, Task<Message>) {
        let task = if config.backend.is_some() {
            Task::done(Message::Connect)
        } else {
            Task::none()
        };
        (
            Self {
                config,
                client: None,
                connected: false,
                error: None,
            },
            task,
        )
    }

    fn finish(&mut self) -> Action {
        self.config.save();
        Action::Return(self.config.clone(), self.client.take().unwrap())
    }

    pub fn update(&mut self, message: Message) -> Action {
        self.error = None;
        match message {
            Message::BackendSet(value) => {
                self.config.backend = Some(value);
                Action::none()
            }
            Message::NetworkSelect(value) => {
                match self.config.backend.as_mut() {
                    Some(ConfigBackend::Embedded { network })
                    | Some(ConfigBackend::Bitcoind { network, .. })
                    | Some(ConfigBackend::Spaced { network, .. }) => *network = value,
                    _ => unreachable!(),
                }
                Action::none()
            }
            Message::UrlInput(value) => {
                match self.config.backend.as_mut() {
                    Some(ConfigBackend::Bitcoind { url, .. })
                    | Some(ConfigBackend::Spaced { url, .. }) => *url = value,
                    _ => unreachable!(),
                }
                Action::none()
            }
            Message::CookieInput(value) => {
                match self.config.backend.as_mut() {
                    Some(ConfigBackend::Bitcoind { cookie, .. }) => *cookie = value,
                    _ => unreachable!(),
                }
                Action::none()
            }
            Message::UserInput(value) => {
                match self.config.backend.as_mut() {
                    Some(ConfigBackend::Bitcoind { user, .. }) => *user = value,
                    _ => unreachable!(),
                }
                Action::none()
            }
            Message::PasswordInput(value) => {
                match self.config.backend.as_mut() {
                    Some(ConfigBackend::Bitcoind { password, .. }) => *password = value,
                    _ => unreachable!(),
                }
                Action::none()
            }
            Message::Connect => match self.config.backend.as_ref() {
                Some(ConfigBackend::Embedded { .. }) => unimplemented!(),
                Some(ConfigBackend::Bitcoind { .. }) => unimplemented!(),
                Some(ConfigBackend::Spaced { url, .. }) => match Client::new(url) {
                    Ok(client) => {
                        let task = client.get_server_info().map(Message::ConnectResult);
                        self.client = Some(client);
                        Action::Task(task)
                    }
                    Err(err) => Action::Task(Task::done(Message::ConnectResult(Err(err)))),
                },
                _ => unreachable!(),
            },
            Message::ConnectResult(result) => match result {
                Ok(info) => {
                    let network = match self.config.backend.as_ref() {
                        Some(ConfigBackend::Embedded { network, .. })
                        | Some(ConfigBackend::Bitcoind { network, .. })
                        | Some(ConfigBackend::Spaced { network, .. }) => network,
                        _ => unreachable!(),
                    };
                    if info.network == network.to_string() {
                        self.config.wallet = None;
                        Action::Task(
                            self.client
                                .as_ref()
                                .unwrap()
                                .list_wallets()
                                .map(Message::ListWalletsResult),
                        )
                    } else {
                        self.client = None;
                        self.error = Some("Wrong network".to_string());
                        Action::none()
                    }
                }
                Err(err) => {
                    self.client = None;
                    self.error = Some(err);
                    Action::none()
                }
            },
            Message::ListWalletsResult(result) => match result {
                Ok(wallets) => {
                    if wallets.is_empty() {
                        self.connected = true;
                        Action::none()
                    } else {
                        if self.config.wallet.is_none() && wallets.contains(&"default".to_string())
                        {
                            self.config.wallet = Some("default".to_string());
                        }
                        self.finish()
                    }
                }
                Err(err) => {
                    self.client = None;
                    self.error = Some(err);
                    Action::none()
                }
            },
            Message::Reset => {
                self.config.backend = None;
                self.client = None;
                self.connected = false;
                Action::none()
            }
            Message::Disconnect => {
                self.client = None;
                self.connected = false;
                Action::none()
            }
            Message::CreateWallet => Action::Task(
                self.client
                    .as_ref()
                    .unwrap()
                    .create_wallet("default".to_string())
                    .map(|r| Message::SetWalletResult(r.result.map(|_| r.label))),
            ),
            Message::ImportWallet => Action::Task(Task::perform(
                async move {
                    let result = rfd::AsyncFileDialog::new()
                        .add_filter("wallet file", &["json"])
                        .pick_file()
                        .await;
                    match result {
                        Some(file) => tokio::fs::read_to_string(file.path())
                            .await
                            .map_err(|e| e.to_string()),
                        None => Err("No file selected".to_string()),
                    }
                },
                Message::ImportWalletPicked,
            )),
            Message::ImportWalletPicked(result) => match result {
                Ok(contents) => Action::Task(
                    self.client
                        .as_ref()
                        .unwrap()
                        .import_wallet(&contents)
                        .map(Message::SetWalletResult),
                ),
                Err(err) => {
                    self.error = Some(err);
                    Action::none()
                }
            },
            Message::SetWalletResult(result) => match result {
                Ok(wallet) => {
                    self.config.wallet = Some(wallet);
                    self.finish()
                }
                Err(err) => {
                    self.error = Some(err);
                    Action::none()
                }
            },
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(if self.config.backend.is_none() {
            column![
                text_big("Select backend"),
                row![
                    column![
                        text_icon(Icon::Assembly).size(150),
                        text_bold("Use embedded light bitcoin node"),
                        submit_button(
                            "Continue",
                            Some(Message::BackendSet(ConfigBackend::Embedded {
                                network: ExtendedNetwork::Mainnet
                            }))
                        ),
                    ]
                    .align_x(Center)
                    .spacing(30),
                    column![
                        text_icon(Icon::CurrencyBitcoin).size(150),
                        text_bold("Connect your own bitcoind"),
                        submit_button(
                            "Continue",
                            Some(Message::BackendSet(ConfigBackend::Bitcoind {
                                network: ExtendedNetwork::Mainnet,
                                url: "http://127.0.0.1:8332".to_string(),
                                cookie: String::new(),
                                user: String::new(),
                                password: String::new(),
                            }))
                        ),
                    ]
                    .align_x(Center)
                    .spacing(30),
                    column![
                        text_icon(Icon::At).size(150),
                        text_bold("Connect your own spaced"),
                        submit_button(
                            "Continue",
                            Some(Message::BackendSet(ConfigBackend::Spaced {
                                network: ExtendedNetwork::Mainnet,
                                url: "http://127.0.0.1:7225".to_string(),
                            }))
                        ),
                    ]
                    .align_x(Center)
                    .spacing(30),
                ]
                .spacing(200),
            ]
            .spacing(10)
        } else if !self.connected {
            column![
                row![
                    button_icon(Icon::ChevronLeft)
                        .style(button::text)
                        .on_press(Message::Reset),
                    text_big("Configure backend"),
                ]
                .align_y(Center),
                error_block(self.error.as_ref()),
                {
                    let networks = [
                        ExtendedNetwork::Mainnet,
                        ExtendedNetwork::Testnet4,
                        ExtendedNetwork::Regtest,
                    ];
                    match self.config.backend.as_ref().unwrap() {
                        ConfigBackend::Embedded { network } => Form::new(
                            "Connect",
                            if self.client.is_none() {
                                Some(Message::Connect)
                            } else {
                                None
                            },
                        )
                        .add_pick_list("Chain", networks, Some(network), Message::NetworkSelect),
                        ConfigBackend::Bitcoind {
                            network,
                            url,
                            cookie,
                            user,
                            password,
                        } => Form::new(
                            "Connect",
                            if self.client.is_none() && !url.is_empty() {
                                Some(Message::Connect)
                            } else {
                                None
                            },
                        )
                        .add_text_input(
                            "Bitcoind JSON-RPC URL",
                            "http://127.0.0.1:7225",
                            url,
                            Message::UrlInput,
                        )
                        .add_text_input("Auth cookie", "none", cookie, Message::CookieInput)
                        .add_text_input("User login", "none", user, Message::UserInput)
                        .add_text_input("User password", "none", password, Message::PasswordInput)
                        .add_pick_list(
                            "Chain",
                            networks,
                            Some(network),
                            Message::NetworkSelect,
                        ),
                        ConfigBackend::Spaced { network, url } => Form::new(
                            "Connect",
                            if self.client.is_none() && !url.is_empty() {
                                Some(Message::Connect)
                            } else {
                                None
                            },
                        )
                        .add_text_input(
                            "Spaced JSON-RPC URL",
                            "http://127.0.0.1:8332",
                            url,
                            Message::UrlInput,
                        )
                        .add_pick_list("Chain", networks, Some(network), Message::NetworkSelect),
                    }
                },
            ]
            .spacing(10)
        } else {
            column![
                row![
                    button_icon(Icon::ChevronLeft)
                        .style(button::text)
                        .on_press(Message::Disconnect),
                    text_big("Set up wallet"),
                ]
                .align_y(Center),
                error_block(self.error.as_ref()),
                row![
                    horizontal_space(),
                    column![
                        text_icon(Icon::NewSection).size(150),
                        text_bold("Create a new spaces wallet"),
                        submit_button("Continue", Some(Message::CreateWallet)),
                    ]
                    .align_x(Center)
                    .spacing(30),
                    column![
                        text_icon(Icon::FolderDown).size(150),
                        text_bold("Load an existing spaces wallet"),
                        submit_button("Continue", Some(Message::ImportWallet)),
                    ]
                    .align_x(Center)
                    .spacing(30),
                    horizontal_space(),
                ]
                .spacing(200),
            ]
            .spacing(10)
        })
        .padding([60, 100])
        .into()
    }
}
