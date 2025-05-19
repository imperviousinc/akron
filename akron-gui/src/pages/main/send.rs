use iced::widget::column;
use iced::Element;

use crate::widget::base::{base_container, result_column};
use crate::widget::tx_result::{TxListMessage, TxResultWidget};
use crate::{
    client::*,
    helpers::*,
    widget::{form::Form, tabs::TabsRow, text::text_big},
};

#[derive(Debug)]
pub struct State {
    asset_kind: AddressKind,
    recipient: String,
    amount: String,
    slabel: Option<SLabel>,
    error: Option<String>,
    tx_result: Option<TxResultWidget>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            asset_kind: AddressKind::Coin,
            recipient: Default::default(),
            amount: Default::default(),
            slabel: Default::default(),
            error: Default::default(),
            tx_result: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TabPress(AddressKind),
    RecipientInput(String),
    AmountInput(String),
    SLabelSelect(SLabel),
    SendCoinsSubmit,
    SendSpaceSubmit,
    ClientResult(Result<WalletResponse, String>),
    TxResult(TxListMessage),
}

pub enum Action {
    None,
    SendCoins { recipient: String, amount: Amount },
    SendSpace { recipient: String, slabel: SLabel },
    ShowTransactions,
}

impl State {
    pub fn reset_inputs(&mut self) {
        self.recipient = Default::default();
        self.amount = Default::default();
        self.slabel = Default::default();
    }

    pub fn update(&mut self, message: Message) -> Action {
        self.error = None;
        self.tx_result = None;

        match message {
            Message::TabPress(asset_kind) => {
                self.asset_kind = asset_kind;
                self.amount = Default::default();
                self.slabel = Default::default();
                Action::None
            }
            Message::RecipientInput(recipient) => {
                if is_recipient_input(&recipient) {
                    self.recipient = recipient;
                }
                Action::None
            }
            Message::AmountInput(amount) => {
                if is_amount_input(&amount) {
                    self.amount = amount
                }
                Action::None
            }
            Message::SLabelSelect(slabel) => {
                self.slabel = Some(slabel);
                Action::None
            }
            Message::SendCoinsSubmit => Action::SendCoins {
                recipient: recipient_from_str(&self.recipient).unwrap(),
                amount: amount_from_str(&self.amount).unwrap(),
            },
            Message::SendSpaceSubmit => Action::SendSpace {
                slabel: self.slabel.clone().unwrap(),
                recipient: recipient_from_str(&self.recipient).unwrap(),
            },
            Message::ClientResult(Ok(w)) => {
                if w.result.iter().any(|r| r.error.is_some()) {
                    self.tx_result = Some(TxResultWidget::new(w));
                    return Action::None;
                }
                self.reset_inputs();
                Action::ShowTransactions
            }
            Message::ClientResult(Err(err)) => {
                self.error = Some(err);
                Action::None
            }
            Message::TxResult(msg) => {
                if let Some(tx_result) = &mut self.tx_result {
                    tx_result.update(msg);
                }
                Action::None
            }
        }
    }

    pub fn view<'a>(&'a self, owned_spaces: &'a Vec<SLabel>) -> Element<'a, Message> {
        base_container(
            column![
                TabsRow::new()
                    .add_tab(
                        "Coins",
                        matches!(self.asset_kind, AddressKind::Coin),
                        Message::TabPress(AddressKind::Coin)
                    )
                    .add_tab(
                        "Spaces",
                        matches!(self.asset_kind, AddressKind::Space),
                        Message::TabPress(AddressKind::Space)
                    ),
                match self.asset_kind {
                    AddressKind::Coin => column![
                        text_big("Send Bitcoin"),
                        result_column(
                            self.error.as_ref(),
                            self.tx_result
                                .as_ref()
                                .map(|tx| TxResultWidget::view(tx).map(Message::TxResult)),
                            [Form::new(
                                "Send",
                                (recipient_from_str(&self.recipient).is_some()
                                    && amount_from_str(&self.amount).is_some())
                                .then_some(Message::SendCoinsSubmit),
                            )
                            .add_text_input("Amount", "sat", &self.amount, Message::AmountInput)
                            .add_text_input(
                                "To",
                                "bitcoin address or @space",
                                &self.recipient,
                                Message::RecipientInput,
                            )
                            .into()]
                        ),
                    ],
                    AddressKind::Space => column![
                        text_big("Send space"),
                        result_column(
                            self.error.as_ref(),
                            self.tx_result
                                .as_ref()
                                .map(|tx| TxResultWidget::view(tx).map(Message::TxResult)),
                            [Form::new(
                                "Send",
                                (recipient_from_str(&self.recipient).is_some()
                                    && self.slabel.is_some())
                                .then_some(Message::SendSpaceSubmit),
                            )
                            .add_pick_list(
                                "Space",
                                owned_spaces.as_slice(),
                                self.slabel.as_ref(),
                                Message::SLabelSelect
                            )
                            .add_text_input(
                                "To",
                                "bitcoin address or @space",
                                &self.recipient,
                                Message::RecipientInput,
                            )
                            .into()]
                        ),
                    ],
                }
                .spacing(40)
            ]
            .spacing(40),
        )
        .into()
    }
}
