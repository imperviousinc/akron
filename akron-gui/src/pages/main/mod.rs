mod home;
mod market;
mod receive;
mod send;
mod settings;
mod sign;
mod spaces;
mod state;

use iced::{
    clipboard, time,
    widget::{
        button, center, column, container, progress_bar, row, text, vertical_rule, vertical_space,
        Column, Stack,
    },
    Center, Color, Element, Fill, Font, Padding, Subscription, Task, Theme,
};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

use crate::{
    client::*,
    widget::{
        fee_rate::{FeeRateMessage, FeeRateSelector},
        icon::{text_icon, Icon},
        text::text_small,
    },
    Config,
};
use iced::widget::button::Status;
use iced::widget::{horizontal_rule, scrollable, stack};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Send,
    Receive,
    Spaces,
    Market,
    Sign,
    Settings,
}

#[derive(Debug)]
pub struct State {
    config: Config,
    client: Client,
    screen: Screen,
    tip_height: u32,
    wallets: state::WalletsCollection,
    spaces: state::SpacesCollection,
    home_screen: home::State,
    send_screen: send::State,
    receive_screen: receive::State,
    spaces_screen: spaces::State,
    market_screen: market::State,
    sign_screen: sign::State,
    settings_screen: settings::State,
    log_buffer: ConstGenericRingBuffer<String, 100>,
    logs_expanded: bool,
    fee_rate_selector: FeeRateSelector,
    fee_rate: Option<FeeRate>,
    fee_rate_confirmed_message: Option<Message>,
}

#[derive(Debug, Clone)]
pub enum Route {
    Home,
    Transactions,
    Send,
    Receive,
    Spaces,
    Space(SLabel),
    Market,
    Sign,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ToggleLogs,
    LogReceived(String),
    NavigateTo(Route),
    ServerInfo(ClientResult<ServerInfo>),
    ListWallets(ClientResult<Vec<String>>),
    WalletLoad(WalletResult<()>),
    WalletInfo(WalletResult<WalletInfoWithProgress>),
    WalletBalance(WalletResult<Balance>),
    WalletSpaces(WalletResult<ListSpacesResponse>),
    WalletTransactions(WalletResult<Vec<TxInfo>>),
    WalletAddress(WalletResult<(AddressKind, String)>),
    SpaceInfo(ClientResult<(SLabel, Option<FullSpaceOut>)>),
    HomeScreen(home::Message),
    SendScreen(send::Message),
    ReceiveScreen(receive::Message),
    SpacesScreen(spaces::Message),
    MarketScreen(market::Message),
    SignScreen(sign::Message),
    SettingsScreen(settings::Message),

    // Fee rate modal
    ShowFeeRateModal,
    FeeRateSelector(FeeRateMessage),
    FeeRateConfirmed(u32),
}

pub enum Action {
    Return(Config),
    Task(Task<Message>),
}

impl State {
    pub fn run(config: Config, client: Client) -> (Self, Task<Message>) {
        let state = Self {
            config,
            client,
            screen: Screen::Home,
            tip_height: 0,
            wallets: Default::default(),
            spaces: Default::default(),
            home_screen: Default::default(),
            send_screen: Default::default(),
            receive_screen: Default::default(),
            spaces_screen: Default::default(),
            market_screen: Default::default(),
            sign_screen: Default::default(),
            settings_screen: Default::default(),
            log_buffer: Default::default(),
            logs_expanded: false,
            fee_rate_selector: Default::default(),
            fee_rate: None,
            fee_rate_confirmed_message: None,
        };
        let task = Task::batch([state.get_server_info(), state.list_wallets()]);
        (state, task)
    }

    fn get_server_info(&self) -> Task<Message> {
        self.client.get_server_info().map(Message::ServerInfo)
    }

    fn list_wallets(&self) -> Task<Message> {
        self.client.list_wallets().map(Message::ListWallets)
    }

    fn get_wallet_info(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            self.client
                .get_wallet_info(wallet.label.to_string())
                .map(Message::WalletInfo)
        } else {
            Task::none()
        }
    }

    fn get_wallet_balance(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            self.client
                .get_wallet_balance(wallet.label.to_string())
                .map(Message::WalletBalance)
        } else {
            Task::none()
        }
    }

    fn get_wallet_spaces(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            self.client
                .get_wallet_spaces(wallet.label.to_string())
                .map(Message::WalletSpaces)
        } else {
            Task::none()
        }
    }

    fn get_wallet_transactions(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            self.client
                .get_wallet_transactions(
                    wallet.label.to_string(),
                    self.home_screen.get_transactions_limit(),
                )
                .map(Message::WalletTransactions)
        } else {
            Task::none()
        }
    }

    fn get_wallet_address(&self, address_kind: AddressKind) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            self.client
                .get_wallet_address(wallet.label.to_string(), address_kind)
                .map(Message::WalletAddress)
        } else {
            Task::none()
        }
    }

    fn get_space_info(&self, slabel: SLabel) -> Task<Message> {
        self.client.get_space_info(slabel).map(Message::SpaceInfo)
    }

    fn navigate_to(&mut self, route: Route) -> Task<Message> {
        match route {
            Route::Home => {
                if self.screen == Screen::Home {
                    self.home_screen.reset();
                } else {
                    self.screen = Screen::Home;
                }
                Task::batch([
                    self.get_wallet_balance(),
                    self.get_wallet_spaces(),
                    self.get_wallet_transactions(),
                ])
            }
            Route::Transactions => {
                self.home_screen.reset();
                self.navigate_to(Route::Home)
            }
            Route::Send => {
                self.screen = Screen::Send;
                self.get_wallet_spaces()
            }
            Route::Receive => {
                self.screen = Screen::Receive;
                Task::batch([
                    self.get_wallet_address(AddressKind::Coin),
                    self.get_wallet_address(AddressKind::Space),
                ])
            }
            Route::Spaces => {
                if self.screen == Screen::Spaces {
                    self.spaces_screen.reset();
                } else {
                    self.screen = Screen::Spaces;
                }
                if let Some(slabel) = self.spaces_screen.get_slabel() {
                    self.get_space_info(slabel)
                } else {
                    self.get_wallet_spaces()
                }
            }
            Route::Space(slabel) => {
                self.screen = Screen::Spaces;
                self.spaces_screen.set_slabel(&slabel);
                self.get_space_info(slabel)
            }
            Route::Market => {
                self.screen = Screen::Market;
                self.get_wallet_spaces()
            }
            Route::Sign => {
                self.screen = Screen::Sign;
                self.get_wallet_spaces()
            }
            Route::Settings => {
                self.screen = Screen::Settings;
                Task::none()
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Tick => {
                let mut tasks = vec![self.get_server_info(), self.get_wallet_info()];
                match self.screen {
                    Screen::Home => {
                        tasks.push(self.get_wallet_balance());
                        tasks.push(self.get_wallet_transactions());
                    }
                    Screen::Spaces => {
                        tasks.push(self.get_wallet_spaces());
                        if let Some(slabel) = self.spaces_screen.get_slabel() {
                            tasks.push(self.get_space_info(slabel));
                        }
                    }
                    _ => {}
                }
                Action::Task(Task::batch(tasks))
            }
            Message::LogReceived(log) => {
                self.log_buffer.push(log);
                Action::Task(Task::none())
            }
            Message::NavigateTo(route) => Action::Task(self.navigate_to(route)),
            Message::ServerInfo(result) => {
                if let Ok(server_info) = result {
                    self.tip_height = server_info.chain.headers;
                }
                Action::Task(Task::none())
            }
            Message::ListWallets(result) => Action::Task(match result {
                Ok(wallets_names) => {
                    self.wallets.set_wallets(&wallets_names);
                    if self.wallets.get_current().is_none() {
                        if let Some(name) = self.config.wallet.as_ref() {
                            self.wallets.set_current(name);
                        }
                    }
                    if let Some(wallet) = self.wallets.get_current() {
                        self.client
                            .load_wallet(wallet.label.clone())
                            .map(Message::WalletLoad)
                    } else {
                        self.navigate_to(Route::Settings)
                    }
                }
                Err(_) => self.list_wallets(),
            }),
            Message::WalletLoad(result) => Action::Task(if result.result.is_ok() {
                Task::batch([self.get_wallet_info(), self.navigate_to(Route::Home)])
            } else {
                Task::none()
            }),
            Message::WalletInfo(WalletResult {
                label: wallet,
                result,
            }) => {
                if let Ok(wallet_info) = result {
                    if let Some(wallet_state) = self.wallets.get_data_mut(&wallet) {
                        wallet_state.info = Some(wallet_info);
                    }
                }
                Action::Task(Task::none())
            }
            Message::WalletBalance(WalletResult {
                label: wallet,
                result,
            }) => {
                if let Ok(balance) = result {
                    if let Some(wallet_state) = self.wallets.get_data_mut(&wallet) {
                        wallet_state.balance = Some(balance.balance);
                    }
                }
                Action::Task(Task::none())
            }
            Message::WalletSpaces(WalletResult {
                label: wallet,
                result,
            }) => {
                if let Ok(spaces) = result {
                    if let Some(wallet_state) = self.wallets.get_data_mut(&wallet) {
                        let mut collect = |spaces: Vec<FullSpaceOut>| -> Vec<SLabel> {
                            spaces
                                .into_iter()
                                .map(|out| {
                                    let name = out.spaceout.space.as_ref().unwrap().name.clone();
                                    self.spaces.set(name.clone(), Some(out));
                                    name
                                })
                                .collect()
                        };
                        wallet_state.pending_spaces = spaces.pending;
                        wallet_state.winning_spaces = collect(spaces.winning);
                        wallet_state.outbid_spaces = collect(spaces.outbid);
                        wallet_state.owned_spaces = collect(spaces.owned);
                    }
                }
                Action::Task(Task::none())
            }
            Message::WalletTransactions(WalletResult {
                label: wallet,
                result,
            }) => {
                if let Ok(transactions) = result {
                    if let Some(wallet_state) = self.wallets.get_data_mut(&wallet) {
                        wallet_state.transactions = transactions;
                    }
                }
                Action::Task(Task::none())
            }
            Message::WalletAddress(WalletResult {
                label: wallet,
                result,
            }) => {
                if let Ok((address_kind, address)) = result {
                    if let Some(wallet_state) = self.wallets.get_data_mut(&wallet) {
                        let address = Some(state::AddressData::new(address));
                        match address_kind {
                            AddressKind::Coin => wallet_state.coin_address = address,
                            AddressKind::Space => wallet_state.space_address = address,
                        }
                    }
                }
                Action::Task(Task::none())
            }
            Message::SpaceInfo(result) => {
                if let Ok((slabel, out)) = result {
                    self.spaces.set(slabel, out)
                }
                Action::Task(Task::none())
            }
            Message::HomeScreen(message) => Action::Task(match self.home_screen.update(message) {
                home::Action::WriteClipboard(s) => clipboard::write(s),
                home::Action::ShowSpace { slabel } => self.navigate_to(Route::Space(slabel)),
                home::Action::GetTransactions => self.get_wallet_transactions(),
                home::Action::BumpFee { txid, fee_rate } => self
                    .client
                    .bump_fee(
                        self.wallets.get_current().unwrap().label.clone(),
                        txid,
                        fee_rate,
                    )
                    .map(|r| Message::HomeScreen(home::Message::BumpFeeResult(r.result))),
                home::Action::None => Task::none(),
            }),
            Message::SendScreen(message) => Action::Task(match self.send_screen.update(message) {
                send::Action::SendCoins { recipient, amount } => {
                    if self.fee_rate.is_none() {
                        self.fee_rate_confirmed_message =
                            Some(Message::SendScreen(send::Message::SendCoinsSubmit));
                        return Action::Task(Task::done(Message::ShowFeeRateModal));
                    }

                    self.client
                        .send_coins(
                            self.wallets.get_current().unwrap().label.clone(),
                            recipient,
                            amount,
                            self.fee_rate.take(),
                        )
                        .map(|r| Message::SendScreen(send::Message::ClientResult(r.result)))
                }
                send::Action::SendSpace { recipient, slabel } => {
                    if self.fee_rate.is_none() {
                        self.fee_rate_confirmed_message =
                            Some(Message::SendScreen(send::Message::SendSpaceSubmit));
                        return Action::Task(Task::done(Message::ShowFeeRateModal));
                    }

                    self.client
                        .send_space(
                            self.wallets.get_current().unwrap().label.clone(),
                            recipient,
                            slabel,
                            self.fee_rate.take(),
                        )
                        .map(|r| Message::SendScreen(send::Message::ClientResult(r.result)))
                }
                send::Action::ShowTransactions => self.navigate_to(Route::Transactions),
                send::Action::None => Task::none(),
            }),
            Message::ReceiveScreen(message) => {
                Action::Task(match self.receive_screen.update(message) {
                    receive::Action::WriteClipboard(s) => clipboard::write(s),
                    receive::Action::None => Task::none(),
                })
            }
            Message::SpacesScreen(message) => {
                Action::Task(match self.spaces_screen.update(message) {
                    spaces::Action::WriteClipboard(s) => clipboard::write(s),
                    spaces::Action::GetSpaceInfo { slabel } => self.get_space_info(slabel),
                    spaces::Action::OpenSpace { slabel, amount } => {
                        if self.fee_rate.is_none() {
                            self.fee_rate_confirmed_message =
                                Some(Message::SpacesScreen(spaces::Message::OpenSubmit));
                            return Action::Task(Task::done(Message::ShowFeeRateModal));
                        }
                        self.client
                            .open_space(
                                self.wallets.get_current().unwrap().label.clone(),
                                slabel,
                                amount,
                                self.fee_rate.take(),
                            )
                            .map(|r| Message::SpacesScreen(spaces::Message::ClientResult(r.result)))
                    }
                    spaces::Action::BidSpace { slabel, amount } => {
                        if self.fee_rate.is_none() {
                            self.fee_rate_confirmed_message =
                                Some(Message::SpacesScreen(spaces::Message::BidSubmit));
                            return Action::Task(Task::done(Message::ShowFeeRateModal));
                        }
                        self.client
                            .bid_space(
                                self.wallets.get_current().unwrap().label.clone(),
                                slabel,
                                amount,
                                self.fee_rate.take(),
                            )
                            .map(|r| Message::SpacesScreen(spaces::Message::ClientResult(r.result)))
                    }
                    spaces::Action::RegisterSpace { slabel } => {
                        if self.fee_rate.is_none() {
                            self.fee_rate_confirmed_message =
                                Some(Message::SpacesScreen(spaces::Message::RegisterSubmit));
                            return Action::Task(Task::done(Message::ShowFeeRateModal));
                        }
                        self.client
                            .register_space(
                                self.wallets.get_current().unwrap().label.clone(),
                                slabel,
                                self.fee_rate.take(),
                            )
                            .map(|r| Message::SpacesScreen(spaces::Message::ClientResult(r.result)))
                    }
                    spaces::Action::RenewSpace { slabel } => {
                        if self.fee_rate.is_none() {
                            self.fee_rate_confirmed_message =
                                Some(Message::SpacesScreen(spaces::Message::RenewSubmit));
                            return Action::Task(Task::done(Message::ShowFeeRateModal));
                        }
                        self.client
                            .renew_space(
                                self.wallets.get_current().unwrap().label.clone(),
                                slabel,
                                self.fee_rate.take(),
                            )
                            .map(|r| Message::SpacesScreen(spaces::Message::ClientResult(r.result)))
                    }
                    spaces::Action::ShowTransactions => self.navigate_to(Route::Transactions),
                    spaces::Action::None => Task::none(),
                })
            }
            Message::MarketScreen(message) => {
                Action::Task(match self.market_screen.update(message) {
                    market::Action::Buy { listing } => {
                        if self.fee_rate.is_none() {
                            self.fee_rate_confirmed_message =
                                Some(Message::MarketScreen(market::Message::BuySubmit));
                            return Action::Task(Task::done(Message::ShowFeeRateModal));
                        }
                        self.client
                            .buy_space(
                                self.wallets.get_current().unwrap().label.clone(),
                                listing,
                                self.fee_rate.take(),
                            )
                            .map(|r| Message::MarketScreen(market::Message::BuyResult(r.result)))
                    }
                    market::Action::Sell { slabel, price } => self
                        .client
                        .sell_space(
                            self.wallets.get_current().unwrap().label.clone(),
                            slabel,
                            price,
                        )
                        .map(|r| Message::MarketScreen(market::Message::SellResult(r.result))),
                    market::Action::WriteClipboard(s) => clipboard::write(s),
                    market::Action::ShowTransactions => self.navigate_to(Route::Transactions),
                    market::Action::None => Task::none(),
                })
            }
            Message::SignScreen(message) => Action::Task(match self.sign_screen.update(message) {
                sign::Action::FilePick => Task::future(async move {
                    let path = rfd::AsyncFileDialog::new()
                        .add_filter("JSON event", &["json"])
                        .pick_file()
                        .await
                        .map(|file| file.path().to_path_buf());

                    let result = if let Some(path) = path {
                        match tokio::fs::read_to_string(&path).await {
                            Ok(content) => match serde_json::from_str::<NostrEvent>(&content) {
                                Ok(event) => Ok(Some((path.to_string_lossy().to_string(), event))),
                                Err(err) => Err(format!("Failed to parse JSON: {}", err)),
                            },
                            Err(err) => Err(format!("Failed to read file: {}", err)),
                        }
                    } else {
                        Ok(None)
                    };
                    Message::SignScreen(sign::Message::EventFileLoaded(result))
                }),
                sign::Action::Sign(slabel, event) => self
                    .client
                    .sign_event(
                        self.wallets.get_current().unwrap().label.clone(),
                        slabel,
                        event,
                    )
                    .then(|result| {
                        let result = result.result;
                        Task::future(async move {
                            let result = match result {
                                Ok(event) => {
                                    let file_path = rfd::AsyncFileDialog::new()
                                        .add_filter("JSON event", &["json"])
                                        .add_filter("All files", &["*"])
                                        .save_file()
                                        .await
                                        .map(|file| file.path().to_path_buf());

                                    if let Some(file_path) = file_path {
                                        use spaces_wallet::bdk_wallet::serde_json;
                                        let contents = serde_json::to_vec(&event).unwrap();
                                        tokio::fs::write(&file_path, contents)
                                            .await
                                            .map_err(|e| e.to_string())
                                    } else {
                                        Ok(())
                                    }
                                }
                                Err(err) => Err(err),
                            };
                            Message::SignScreen(sign::Message::EventFileSaved(result))
                        })
                    }),
                sign::Action::None => Task::none(),
            }),
            Message::SettingsScreen(message) => match self.settings_screen.update(message) {
                settings::Action::SetCurrentWallet(name) => {
                    self.wallets.set_current(&name);
                    self.config.wallet = Some(name);
                    self.config.save();
                    Action::Task(self.list_wallets())
                }
                settings::Action::ExportWallet(wallet_name) => {
                    Action::Task(self.client.export_wallet(wallet_name).then(|result| {
                        let result = result.result;
                        Task::future(async move {
                            let result = match result {
                                Ok(contents) => {
                                    let file_path = rfd::AsyncFileDialog::new()
                                        .add_filter("Wallet file", &["json"])
                                        .add_filter("All files", &["*"])
                                        .save_file()
                                        .await
                                        .map(|file| file.path().to_path_buf());

                                    if let Some(file_path) = file_path {
                                        tokio::fs::write(&file_path, contents)
                                            .await
                                            .map_err(|e| e.to_string())
                                    } else {
                                        Ok(())
                                    }
                                }
                                Err(err) => Err(err),
                            };
                            Message::SettingsScreen(settings::Message::WalletFileSaved(result))
                        })
                    }))
                }
                settings::Action::CreateWallet(wallet_name) => {
                    self.config.wallet = None;
                    self.wallets.unset_current();
                    Action::Task(
                        self.client
                            .create_wallet(wallet_name)
                            .map(|r| {
                                Message::SettingsScreen(settings::Message::WalletCreated(r.result))
                            })
                            .chain(self.list_wallets()),
                    )
                }
                settings::Action::FilePick => Action::Task(
                    Task::future(async move {
                        let result = rfd::AsyncFileDialog::new()
                            .add_filter("wallet file", &["json"])
                            .pick_file()
                            .await;
                        match result {
                            Some(file) => tokio::fs::read_to_string(file.path()).await.ok(),
                            None => None,
                        }
                    })
                    .map(|r| Message::SettingsScreen(settings::Message::WalletFileLoaded(r))),
                ),
                settings::Action::ImportWallet(contents) => {
                    self.config.wallet = None;
                    self.wallets.unset_current();
                    Action::Task(
                        self.client
                            .import_wallet(&contents)
                            .map(|r| {
                                Message::SettingsScreen(settings::Message::WalletFileImported(
                                    r.map(|_| ()),
                                ))
                            })
                            .chain(self.list_wallets()),
                    )
                }
                settings::Action::ResetBackend => {
                    self.config.remove();
                    Action::Return(self.config.clone())
                }
                settings::Action::None => Action::Task(Task::none()),
            },
            Message::ToggleLogs => {
                self.logs_expanded = !self.logs_expanded;
                Action::Task(Task::none())
            }
            // Fee rate modal
            Message::ShowFeeRateModal => Action::Task(
                self.fee_rate_selector
                    .update(FeeRateMessage::ShowModal)
                    .map(Message::FeeRateSelector),
            ),
            Message::FeeRateSelector(msg) => {
                let task = self.fee_rate_selector.update(msg.clone());
                Action::Task(match msg {
                    FeeRateMessage::Confirmed(fee_rate) => Task::batch(vec![
                        task.map(Message::FeeRateSelector),
                        Task::done(Message::FeeRateConfirmed(fee_rate)),
                    ]),
                    _ => task.map(Message::FeeRateSelector),
                })
            }
            Message::FeeRateConfirmed(fee_rate) => {
                self.fee_rate = FeeRate::from_sat_per_vb(fee_rate as _);

                if let Some(msg) = self.fee_rate_confirmed_message.take() {
                    return Action::Task(Task::done(msg));
                }
                Action::Task(Task::none())
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = self.main_view();
        stack![
            content,
            self.fee_rate_selector.view().map(Message::FeeRateSelector)
        ]
        .into()
    }

    pub fn main_view(&self) -> Element<'_, Message> {
        let navbar_button = |label, icon: Icon, route: Route, screen: Screen| {
            let button = button(
                row![
                    if self.screen == screen {
                        text_icon(icon).size(20).style(text::primary)
                    } else {
                        text_icon(icon).size(20)
                    },
                    text(label).size(16)
                ]
                .spacing(10)
                .align_y(Center),
            )
            .style(move |theme: &Theme, status: button::Status| {
                let mut style = if self.screen == screen {
                    button::secondary
                } else {
                    button::text
                }(theme, status);
                style.border = style.border.rounded(4);
                style
            })
            .width(Fill);
            button.on_press(Message::NavigateTo(route))
        };

        Column::new()
            .push(row![
                // SIDEBAR
                column![
                    navbar_button("Home", Icon::Bitcoin, Route::Home, Screen::Home,),
                    navbar_button("Send", Icon::ArrowBigUpDash, Route::Send, Screen::Send,),
                    navbar_button(
                        "Receive",
                        Icon::ArrowBigDownDash,
                        Route::Receive,
                        Screen::Receive,
                    ),
                    navbar_button("Spaces", Icon::AtSign, Route::Spaces, Screen::Spaces,),
                    navbar_button("Market", Icon::Store, Route::Market, Screen::Market,),
                    navbar_button("Sign", Icon::UserRoundPen, Route::Sign, Screen::Sign,),
                    vertical_space(),
                    navbar_button(
                        "Settings",
                        Icon::Settings,
                        Route::Settings,
                        Screen::Settings,
                    ),
                ]
                .padding(10)
                .spacing(5)
                .width(200),
                vertical_rule(3),
                Column::new()
                    .height(Fill)
                    .width(Fill)
                    .push_maybe(self.wallets.get_current().and_then(|wallet| {
                        if !wallet.is_synced() {
                            Some(
                                Stack::new()
                                    .push(
                                        container(
                                            progress_bar(
                                                0.0..=1.0,
                                                wallet.sync_status_percentage(),
                                            )
                                            .style(|t| {
                                                let mut style = progress_bar::primary(t);
                                                let p = t.extended_palette();
                                                style.bar = p.primary.weak.color.into();
                                                style
                                            })
                                            .height(Fill),
                                        )
                                        .height(40),
                                    )
                                    .push(center(
                                        text(format!(
                                            "{} ({:.1}%)",
                                            wallet.sync_status_string(),
                                            wallet.sync_status_percentage() * 100.0,
                                        ))
                                        .size(14),
                                    )),
                            )
                        } else {
                            None
                        }
                    }))
                    .push(
                        container(match &self.screen {
                            Screen::Home =>
                                if let Some(wallet) = self.wallets.get_current() {
                                    self.home_screen
                                        .view(
                                            self.tip_height,
                                            wallet.state.balance,
                                            &wallet.state.transactions,
                                        )
                                        .map(Message::HomeScreen)
                                } else {
                                    center("No wallet loaded").into()
                                },
                            Screen::Send =>
                                if let Some(wallet) = self.wallets.get_current() {
                                    self.send_screen
                                        .view(&wallet.state.owned_spaces)
                                        .map(Message::SendScreen)
                                } else {
                                    center("No wallet loaded").into()
                                },
                            Screen::Receive =>
                                if let Some(wallet) = self.wallets.get_current() {
                                    self.receive_screen
                                        .view(
                                            wallet.state.coin_address.as_ref(),
                                            wallet.state.space_address.as_ref(),
                                        )
                                        .map(Message::ReceiveScreen)
                                } else {
                                    center("No wallet loaded").into()
                                },
                            Screen::Spaces =>
                                if let Some(wallet) = self.wallets.get_current() {
                                    self.spaces_screen
                                        .view(
                                            self.tip_height,
                                            &self.spaces,
                                            &wallet.state.pending_spaces,
                                            &wallet.state.winning_spaces,
                                            &wallet.state.outbid_spaces,
                                            &wallet.state.owned_spaces,
                                        )
                                        .map(Message::SpacesScreen)
                                } else {
                                    center("No wallet loaded").into()
                                },
                            Screen::Market =>
                                if let Some(wallet) = self.wallets.get_current() {
                                    self.market_screen
                                        .view(wallet.state.owned_spaces.as_ref())
                                        .map(Message::MarketScreen)
                                } else {
                                    center("No wallet loaded").into()
                                },
                            Screen::Sign =>
                                if let Some(wallet) = self.wallets.get_current() {
                                    self.sign_screen
                                        .view(&wallet.state.owned_spaces)
                                        .map(Message::SignScreen)
                                } else {
                                    center("No wallet loaded").into()
                                },
                            Screen::Settings => self
                                .settings_screen
                                .view(
                                    self.config.backend.as_ref().unwrap().network(),
                                    self.tip_height,
                                    self.wallets.get_wallets(),
                                    self.wallets.get_current().map(|w| w.label),
                                )
                                .map(Message::SettingsScreen),
                        })
                        .height(Fill)
                    )
            ])
            .push_maybe(self.logs_view())
            .into()
    }

    pub fn logs_view(&self) -> Option<Element<'_, Message>> {
        if self.log_buffer.is_empty() {
            return None;
        }

        let toggle_icon = if self.logs_expanded {
            text_icon(Icon::ChevronDown)
        } else {
            text_icon(Icon::ChevronRight)
        };
        let toggle_btn = button(toggle_icon.size(26))
            .padding([0, 10])
            .style(|theme: &Theme, s| {
                let palette = theme.extended_palette();

                let bg = match s {
                    Status::Active => Color::TRANSPARENT.into(),
                    _ => palette.secondary.strong.color.into(),
                };

                button::Style {
                    background: Some(bg),
                    text_color: Color::BLACK,
                    border: Default::default(),
                    shadow: Default::default(),
                }
            })
            .on_press(Message::ToggleLogs);

        let (log_header, logs) = if self.logs_expanded {
            (
                text_small("Status: "),
                Some(
                    container(
                        scrollable(column(
                            self.log_buffer
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
                    .padding(Padding {
                        top: 0.0,
                        right: 10.0,
                        bottom: 10.0,
                        left: 10.0,
                    })
                    .height(280)
                    .width(Fill),
                ),
            )
        } else {
            (
                text_small(
                    self.log_buffer
                        .back()
                        .map(|s| s.to_string())
                        .unwrap_or("".to_string()),
                ),
                None,
            )
        };

        let logs_style = move |theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                text_color: None,
                background: if !self.logs_expanded {
                    Some(palette.background.weak.color.into())
                } else {
                    None
                },
                border: Default::default(),
                shadow: Default::default(),
            }
        };

        let status_row = row![
            log_header,
            iced::widget::Space::with_width(Fill),
            toggle_btn,
        ]
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 10.0,
        })
        .align_y(Center);

        let view = container(
            column![horizontal_rule(3), status_row,]
                .push_maybe(logs)
                .push(horizontal_rule(3))
                .width(Fill),
        )
        .width(Fill)
        .style(logs_style);

        Some(view.into())
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let ticks = time::every(
            if self.tip_height != 0 && self.wallets.get_current().is_some_and(|w| w.is_synced()) {
                time::Duration::from_secs(30)
            } else {
                time::Duration::from_secs(2)
            },
        )
        .map(|_| Message::Tick);

        let logs = self.client.logs_subscription().map(Message::LogReceived);

        let fee_rate = self
            .fee_rate_selector
            .subscription()
            .map(Message::FeeRateSelector);

        Subscription::batch([ticks, logs, fee_rate])
    }
}
