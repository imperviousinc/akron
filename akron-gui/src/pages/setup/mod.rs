use iced::{
    widget::{button, center, column, container, horizontal_rule, row, scrollable, text},
    Bottom, Center, Color, Element, Fill, Font, Shrink, Subscription, Task, Theme,
};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

use spaces_client::config::ExtendedNetwork;
use spaces_protocol::constants::ChainAnchor;

use crate::{
    client::{Client, ClientResult, ServerInfo},
    widget::{
        base::base_container,
        form::{submit_button, Form},
        icon::{button_icon, text_icon, Icon},
        text::{error_block, text_big, text_bold, text_semibold, text_small},
    },
    Config, ConfigBackend,
};

#[derive(Debug)]
pub struct State {
    config: Config,
    client: Option<Client>,
    connecting: bool,
    logs: ConstGenericRingBuffer<String, 100>,
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
    ConnectResult(Result<(Client, ConfigBackend), String>),
    GetServerInfoResult(ClientResult<ServerInfo>),
    ListWalletsResult(ClientResult<Vec<String>>),
    Reset,
    Disconnect,
    CreateWallet,
    ImportWallet,
    ImportWalletPicked(Result<String, String>),
    SetWalletResult(Result<String, String>),
    LogReceived(String),
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
                connecting: false,
                logs: Default::default(),
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
                    Some(ConfigBackend::Akrond { network, .. })
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
            Message::Connect => {
                if self.connecting {
                    return Action::none();
                }
                self.connecting = true;
                let data_dir = self.config.data_dir().to_path_buf();
                let backend_config = self.config.backend.clone().unwrap();
                Action::Task(Task::perform(
                    async move { Client::create(data_dir, backend_config).await },
                    Message::ConnectResult,
                ))
            }
            Message::ConnectResult(result) => match result {
                Ok((client, backend_config)) => {
                    self.client = Some(client);
                    self.config.backend = Some(backend_config);
                    Action::Task(
                        self.client
                            .as_ref()
                            .unwrap()
                            .get_server_info()
                            .map(Message::GetServerInfoResult),
                    )
                }
                Err(err) => {
                    self.connecting = false;
                    self.error = Some(err);
                    Action::none()
                }
            },
            Message::GetServerInfoResult(result) => {
                match result {
                    Ok(server_info) => {
                        let backend_config = self.config.backend.as_ref().unwrap();
                        match backend_config {
                            ConfigBackend::Akrond { .. } => {}
                            ConfigBackend::Bitcoind { network, .. }
                            | ConfigBackend::Spaced { network, .. } => {
                                if server_info.network != network.to_string() {
                                    self.client = None;
                                    self.connecting = false;
                                    self.error = Some("Wrong network".to_string());
                                    return Action::none();
                                }
                            }
                        }
                        if server_info.chain.headers
                            >= (match backend_config {
                                ConfigBackend::Akrond { prune_point, .. } => {
                                    prune_point.map_or(0, |p| p.height)
                                }
                                ConfigBackend::Bitcoind { network, .. }
                                | ConfigBackend::Spaced { network, .. } => match network {
                                    ExtendedNetwork::Mainnet => ChainAnchor::MAINNET().height,
                                    ExtendedNetwork::Testnet4 => ChainAnchor::TESTNET4().height,
                                    _ => 0,
                                },
                            })
                        {
                            return if self.config.wallet.is_none() {
                                Action::Task(
                                    self.client
                                        .as_ref()
                                        .unwrap()
                                        .list_wallets()
                                        .map(Message::ListWalletsResult),
                                )
                            } else {
                                self.finish()
                            };
                        }
                    }
                    Err(err) => {
                        self.logs.push(err);
                    }
                }
                Action::Task(
                    Task::future(tokio::time::sleep(std::time::Duration::from_secs(1)))
                        .discard()
                        .chain(
                            self.client
                                .as_ref()
                                .unwrap()
                                .get_server_info()
                                .map(Message::GetServerInfoResult),
                        ),
                )
            }
            Message::ListWalletsResult(result) => match result {
                Ok(wallets) => {
                    self.connecting = false;
                    if wallets.is_empty() {
                        Action::none()
                    } else {
                        if wallets.contains(&"default".to_string()) {
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
                if self.connecting {
                    return Action::none();
                }
                self.config.backend = None;
                self.client = None;
                Action::none()
            }
            Message::Disconnect => {
                if self.connecting {
                    return Action::none();
                }
                self.client = None;
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
            Message::LogReceived(log) => {
                self.logs.push(log);
                Action::Task(Task::none())
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        const DESCRIPTION_TEXT_HEIGHT: u16 = 100;

        container(if self.config.backend.is_none() {
            column![
                text_big("Select backend"),
                row![
                    column![
                        text_icon(Icon::Bolt).size(150),
                        text_bold("Compact Bitcoin node"),
                        text("Faster checkpointed sync with minimal storage. Syncs essential data from peers. Easiest for most users.")
                        .height(DESCRIPTION_TEXT_HEIGHT),
                        submit_button(
                           text("Start").width(Fill).align_x(Center),
                            Some(Message::BackendSet(ConfigBackend::Akrond {
                                network: ExtendedNetwork::Mainnet,
                                prune_point: None,
                            }))
                        ),
                    ]
                    .align_x(Center)
                    .spacing(30),
                    column![
                        text_icon(Icon::Bitcoin).size(150),
                        text_bold("Full Node"),
                        text("Use your own Bitcoin node. Requires blockchain data not pruned before block 871222.")
                        .height(DESCRIPTION_TEXT_HEIGHT),
                        submit_button(
                            text("Connect").width(Fill).align_x(Center),
                            Some(Message::BackendSet(ConfigBackend::Bitcoind {
                                network: ExtendedNetwork::Mainnet,
                                url: "http://127.0.0.1:8332".to_string(),
                                cookie: String::new(),
                                user: String::new(),
                                password: String::new(),
                            }))
                        ).style(|theme: &Theme, status: button::Status| {
                            let mut style = button::secondary(theme, status);
                            style.border = style.border.rounded(7);
                            style
                        }),
                    ]
                    .align_x(Center)
                    .spacing(30),
                    column![
                        text_icon(Icon::AtSign).size(150),
                        text_bold("Spaced instance"),
                        text("For users running Spaced connected to a Bitcoin node on their own server.")
                        .height(DESCRIPTION_TEXT_HEIGHT),
                        submit_button(
                            text("Connect").width(Fill).align_x(Center),
                            Some(Message::BackendSet(ConfigBackend::Spaced {
                                network: ExtendedNetwork::Mainnet,
                                url: "http://127.0.0.1:7225".to_string(),
                            }))
                        ).style(|theme: &Theme, status: button::Status| {
                            let mut style = button::secondary(theme, status);
                            style.border = style.border.rounded(7);
                            style
                        }),
                    ]
                    .align_x(Center)
                    .spacing(30),
                ].align_y(Bottom).padding([0, 80]).spacing(80)
            ]
            .spacing(10)
        } else if self.connecting {
            column![
                center(text_semibold("Please wait... This may take a few minutes.").size(16)).padding([10, 0]).height(Shrink),
                horizontal_rule(3.0),
                container(
                    scrollable(column(
                        self.logs
                            .iter()
                            .map(|line| {
                                text_small(line.clone())
                                    .color(Color::BLACK)
                                    .font(Font::MONOSPACE)
                                    .into()
                            })
                            .collect::<Vec<_>>(),
                    ))
                    .width(Fill)
                    .height(Fill)
                    .anchor_bottom(),
                )
                .padding(10)
                .height(Fill)
                .width(Fill),
            ]
        } else if self.client.is_none() {
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
                        ConfigBackend::Akrond { network, .. } => {
                            base_container(
                            Form::new("Connect", Some(Message::Connect)).add_pick_list(
                                "Chain",
                                [ExtendedNetwork::Mainnet, ExtendedNetwork::Testnet4],
                                Some(network),
                                Message::NetworkSelect,
                            ))
                        }
                        ConfigBackend::Bitcoind {
                            network,
                            url,
                            cookie,
                            user,
                            password,
                        } => base_container(Form::new("Connect", Some(Message::Connect))
                            .add_text_input(
                                "Bitcoind JSON-RPC URL",
                                "http://127.0.0.1:7225",
                                url,
                                Message::UrlInput,
                            )
                            .add_text_input("Auth cookie", "none", cookie, Message::CookieInput)
                            .add_text_input("User login", "none", user, Message::UserInput)
                            .add_text_input(
                                "User password",
                                "none",
                                password,
                                Message::PasswordInput,
                            )
                            .add_pick_list(
                                "Chain",
                                networks,
                                Some(network),
                                Message::NetworkSelect,
                            )),
                        ConfigBackend::Spaced { network, url } => {
                            base_container(Form::new("Connect", Some(Message::Connect))
                                .add_text_input(
                                    "Spaced JSON-RPC URL",
                                    "http://127.0.0.1:8332",
                                    url,
                                    Message::UrlInput,
                                )
                                .add_pick_list(
                                    "Chain",
                                    networks,
                                    Some(network),
                                    Message::NetworkSelect,
                                ))
                        }
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
                    column![
                        text_icon(Icon::WalletMinimal).size(150),
                        text_semibold("Create a new spaces wallet").size(20),
                        submit_button(text("Continue").align_x(Center).width(Fill), Some(Message::CreateWallet)),
                    ]
                    .align_x(Center)
                    .spacing(30),
                    column![
                        text_icon(Icon::FolderDown).size(150),
                        text_semibold("Load an existing spaces wallet").size(20),
                        submit_button(text("Continue").align_x(Center).width(Fill), Some(Message::ImportWallet)),
                    ]
                    .align_x(Center)
                    .spacing(30),
                ].align_y(Bottom)
                .spacing(140).padding([0, 140]),
            ]
            .spacing(10)
        })
        .padding([60, 100])
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if let Some(client) = self.client.as_ref() {
            client.logs_subscription().map(Message::LogReceived)
        } else {
            Subscription::none()
        }
    }
}
