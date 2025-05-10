use iced::{
    Center, Element, Fill, Shrink,
    widget::{button, column, row, text},
};

use crate::widget::{
    form::{pick_list, submit_button, text_input},
    text::{error_block, text_big},
};

#[derive(Debug, Default)]
pub struct State {
    new_wallet_name: String,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    WalletSelect(String),
    ExportWalletPress(String),
    NewWalletInput(String),
    CreateWalletPress,
    ImportWalletPress,
    ResetBackendPress,
    WalletFileSaved(Result<(), String>),
    WalletCreated(Result<(), String>),
    WalletFileLoaded(Option<String>),
    WalletFileImported(Result<(), String>),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    SetCurrentWallet(String),
    ExportWallet(String),
    CreateWallet(String),
    FilePick,
    ImportWallet(String),
    ResetBackend,
}

impl State {
    pub fn update(&mut self, message: Message) -> Action {
        self.error = None;
        match message {
            Message::WalletSelect(w) => Action::SetCurrentWallet(w),
            Message::ExportWalletPress(w) => Action::ExportWallet(w),
            Message::NewWalletInput(w) => {
                if w.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                    self.new_wallet_name = w;
                }
                Action::None
            }
            Message::CreateWalletPress => Action::CreateWallet(self.new_wallet_name.to_string()),
            Message::ImportWalletPress => Action::FilePick,
            Message::ResetBackendPress => Action::ResetBackend,
            Message::WalletFileSaved(result) | Message::WalletFileImported(result) => {
                if let Err(err) = result {
                    self.error = Some(err);
                }
                Action::None
            }
            Message::WalletFileLoaded(contents) => {
                if let Some(contents) = contents {
                    Action::ImportWallet(contents)
                } else {
                    Action::None
                }
            }
            Message::WalletCreated(result) => {
                if let Err(err) = result {
                    self.error = Some(err);
                } else {
                    self.new_wallet_name = String::new();
                }
                Action::None
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        wallets_names: Vec<&'a String>,
        wallet_name: Option<&'a String>,
    ) -> Element<'a, Message> {
        column![
            column![
                text_big("Wallet"),
                error_block(self.error.as_ref()),
                row![
                    pick_list(wallets_names, wallet_name, |w| {
                        Message::WalletSelect(w.to_string())
                    })
                    .width(Fill),
                    submit_button(
                        "Export",
                        wallet_name.map(|w| Message::ExportWalletPress(w.to_string()))
                    )
                    .width(Shrink),
                ]
                .spacing(20),
                row![
                    text_input("default", &self.new_wallet_name).on_input(Message::NewWalletInput),
                    submit_button(
                        "Create",
                        if self.new_wallet_name.is_empty() {
                            None
                        } else {
                            Some(Message::CreateWalletPress)
                        }
                    ),
                    submit_button("Import", Some(Message::ImportWalletPress)),
                ]
                .spacing(20),
            ]
            .spacing(10),
            column![
                text_big("Backend"),
                button(text("Reset backend settings").align_x(Center).width(Fill))
                    .on_press(Message::ResetBackendPress)
                    .style(button::danger)
                    .padding(10)
                    .width(Fill),
            ]
            .spacing(10)
        ]
        .padding([60, 100])
        .spacing(20)
        .into()
    }
}
