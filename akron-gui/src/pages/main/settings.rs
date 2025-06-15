use crate::widget::base::{base_container, result_column};
use crate::widget::form::STANDARD_PADDING;
use crate::widget::{
    form::{pick_list, submit_button, text_input},
    text::text_big,
};
use iced::border::rounded;
use iced::{
    widget::{button, column, row, text},
    Center, Element, Fill, Shrink, Theme,
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
    WalletCreated(Result<String, String>),
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
        base_container(
            column![
                column![
                    text_big("Wallet"),
                    result_column(
                        self.error.as_ref(),
                        None,
                        [
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
                            .spacing(20)
                            .into(),
                            row![
                                text_input("default", &self.new_wallet_name)
                                    .width(Fill)
                                    .on_input(Message::NewWalletInput),
                                row![
                                    submit_button(
                                        text("Create").align_x(Center),
                                        if self.new_wallet_name.is_empty() {
                                            None
                                        } else {
                                            Some(Message::CreateWalletPress)
                                        }
                                    ),
                                    submit_button(
                                        text("Import").align_x(Center),
                                        Some(Message::ImportWalletPress)
                                    ),
                                ]
                                .spacing(5)
                            ]
                            .spacing(20)
                            .into()
                        ]
                    )
                    .spacing(40),
                ]
                .spacing(40),
                column![
                    text_big("Backend"),
                    button(text("Reset backend settings").align_x(Center).width(Fill))
                        .on_press(Message::ResetBackendPress)
                        .style(|t: &Theme, status: button::Status| {
                            let mut style = button::danger(t, status);
                            let p = t.extended_palette();
                            if matches!(status, button::Status::Active) {
                                style.background = Some(p.danger.weak.color.into());
                            }
                            style.border = rounded(7);
                            style
                        })
                        .padding(STANDARD_PADDING)
                        .width(Fill),
                ]
                .spacing(40)
            ]
            .spacing(40),
        )
        .into()
    }
}
