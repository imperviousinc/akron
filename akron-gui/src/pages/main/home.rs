use serde::Deserialize;
use std::str::FromStr;

use iced::{
    Center, Element, Fill, FillPortion, Theme,
    widget::{
        Column, Row, Space, button, center, column, container, horizontal_rule, horizontal_space,
        row, scrollable, text,
    },
};

use crate::{
    client::*,
    helpers::*,
    widget::{
        form::Form,
        icon::{Icon, button_icon, text_icon},
        text::{error_block, text_big, text_bold, text_monospace, text_monospace_bold, text_small},
    },
};

#[derive(Debug)]
pub struct State {
    txid: Option<Txid>,
    transactions_limit: usize,
    fee_rate: String,
    error: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            txid: None,
            transactions_limit: 10,
            fee_rate: String::new(),
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    BackPress,
    TxidPress(Txid),
    CopyTxidPress(Txid),
    SpacePress(SLabel),
    TxsListScrolled(f32, usize),
    FeeRateInput(String),
    BumpFeeSubmit,
    BumpFeeResult(Result<(), String>),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    WriteClipboard(String),
    ShowSpace { slabel: SLabel },
    GetTransactions,
    BumpFee { txid: Txid, fee_rate: FeeRate },
}

impl State {
    pub fn reset_inputs(&mut self) {
        self.fee_rate = String::new();
    }

    pub fn reset(&mut self) {
        self.txid = None;
        self.reset_inputs();
    }

    pub fn get_transactions_limit(&self) -> usize {
        self.transactions_limit
    }

    pub fn update(&mut self, message: Message) -> Action {
        self.error = None;
        match message {
            Message::BackPress => {
                self.txid = None;
                Action::None
            }
            Message::TxidPress(txid) => {
                self.txid = Some(txid);
                Action::None
            }
            Message::SpacePress(slabel) => Action::ShowSpace { slabel },
            Message::CopyTxidPress(txid) => Action::WriteClipboard(txid.to_string()),
            Message::TxsListScrolled(percentage, count) => {
                if percentage > 0.8 && count >= self.transactions_limit {
                    self.transactions_limit += (percentage * count as f32) as usize;
                    Action::GetTransactions
                } else {
                    Action::None
                }
            }
            Message::FeeRateInput(fee_rate) => {
                if is_fee_rate_input(&fee_rate) {
                    self.fee_rate = fee_rate
                }
                Action::None
            }
            Message::BumpFeeSubmit => Action::BumpFee {
                txid: self.txid.unwrap(),
                fee_rate: fee_rate_from_str(&self.fee_rate).unwrap().unwrap(),
            },
            Message::BumpFeeResult(Ok(())) => {
                self.reset();
                Action::GetTransactions
            }
            Message::BumpFeeResult(Err(err)) => {
                self.error = Some(err);
                Action::None
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        tip_height: u32,
        balance: Amount,
        transactions: &'a [TxInfo],
    ) -> Element<'a, Message> {
        if let Some(txid) = self.txid.as_ref() {
            if let Some(transaction) = transactions.iter().find(|tx| &tx.txid == txid) {
                let event_row_with_space = |action: &'static str,
                                            space: &'a str,
                                            amount: Option<Amount>|
                 -> Row<'a, Message> {
                    let slabel = SLabel::from_str(space).unwrap();
                    row![
                        text(action),
                        button(text_monospace(space))
                            .on_press(Message::SpacePress(slabel))
                            .style(button::text)
                            .padding(0)
                    ]
                    .push_maybe(amount.map(|amount| text(format_amount(amount))))
                };

                let event_row_with_string = |action: &'static str, s: String| -> Row<'a, Message> {
                    row![text(action), text(s)]
                };

                let events_rows: Vec<Element<'a, Message>> = transaction
                    .events
                    .iter()
                    .filter_map(|event| match event {
                        TxEvent {
                            kind: TxEventKind::Commit,
                            space,
                            ..
                        } => Some(event_row_with_space(
                            "Commit",
                            space.as_ref().unwrap(),
                            None,
                        )),
                        TxEvent {
                            kind: TxEventKind::Bidout,
                            details,
                            ..
                        } => Some(event_row_with_string(
                            "Bidout",
                            BidoutEventDetails::deserialize(details.as_ref().unwrap())
                                .unwrap()
                                .count
                                .to_string(),
                        )),
                        TxEvent {
                            kind: TxEventKind::Open,
                            space,
                            details,
                            ..
                        } => Some(event_row_with_space(
                            "Open",
                            space.as_ref().unwrap(),
                            Some(
                                OpenEventDetails::deserialize(details.as_ref().unwrap())
                                    .unwrap()
                                    .initial_bid,
                            ),
                        )),
                        TxEvent {
                            kind: TxEventKind::Bid,
                            space,
                            details,
                            ..
                        } => Some(event_row_with_space(
                            "Bid",
                            space.as_ref().unwrap(),
                            Some(
                                BidEventDetails::deserialize(details.as_ref().unwrap())
                                    .unwrap()
                                    .current_bid,
                            ),
                        )),
                        TxEvent {
                            kind: TxEventKind::Register,
                            space,
                            ..
                        } => Some(event_row_with_space(
                            "Register",
                            space.as_ref().unwrap(),
                            None,
                        )),
                        TxEvent {
                            kind: TxEventKind::Transfer,
                            space,
                            ..
                        } => Some(event_row_with_space(
                            "Transfer",
                            space.as_ref().unwrap(),
                            None,
                        )),
                        TxEvent {
                            kind: TxEventKind::Renew,
                            space,
                            ..
                        } => Some(event_row_with_space("Renew", space.as_ref().unwrap(), None)),
                        TxEvent {
                            kind: TxEventKind::Send,
                            ..
                        } => Some(event_row_with_string(
                            "Send",
                            format_amount(
                                SendEventDetails::deserialize(event.details.as_ref().unwrap())
                                    .unwrap()
                                    .amount,
                            ),
                        )),
                        TxEvent {
                            kind: TxEventKind::Buy,
                            space,
                            ..
                        } => Some(event_row_with_space("Buy", space.as_ref().unwrap(), None)),
                        TxEvent {
                            kind: TxEventKind::FeeBump,
                            ..
                        } => Some(event_row_with_string("Bump fee", String::new())),
                        _ => None,
                    })
                    .map(|row| row.spacing(10).into())
                    .collect();

                column![
                    row![
                        button(text_icon(Icon::ChevronLeft).size(20))
                            .style(button::text)
                            .on_press(Message::BackPress),
                        text_monospace_bold(txid.to_string()).size(20),
                        button_icon(Icon::Copy)
                            .style(button::text)
                            .on_press(Message::CopyTxidPress(*txid)),
                    ]
                    .spacing(5)
                    .align_y(Center),
                    horizontal_rule(3),
                    row![
                        column![
                            text_bold("Info"),
                            text(format!("Sent: {}", format_amount(transaction.sent))),
                            text(format!("Received: {}", format_amount(transaction.received))),
                        ]
                        .push_maybe(
                            transaction
                                .fee
                                .map(|fee| { text(format!("Fee: {}", format_amount(fee))) })
                        )
                        .push_maybe(transaction.block_height.map(|block_height| text(format!(
                            "Block: {} ({})",
                            block_height,
                            height_to_past_est(block_height, tip_height)
                        ))))
                        .push_maybe(if events_rows.is_empty() {
                            None
                        } else {
                            Some(text_bold("Events"))
                        })
                        .extend(events_rows.into_iter())
                        .spacing(10)
                        .width(Fill),
                        if transaction.block_height.is_some() {
                            column![]
                        } else {
                            column![
                                text_big("Bump fee"),
                                error_block(self.error.as_ref()),
                                Form::new(
                                    "Bump fee",
                                    fee_rate_from_str(&self.fee_rate)
                                        .flatten()
                                        .map(|_| Message::BumpFeeSubmit),
                                )
                                .add_text_input(
                                    "Fee rate",
                                    "sat/vB",
                                    &self.fee_rate,
                                    Message::FeeRateInput,
                                ),
                            ]
                            .spacing(10)
                        }
                        .width(Fill)
                    ]
                    .spacing(20)
                ]
                .padding(20)
                .spacing(20)
                .into()
            } else {
                center("Transaction is not found").into()
            }
        } else {
            column![
                column![text_big("Balance"), text(format_amount(balance)),]
                    .padding([30, 0])
                    .spacing(10)
                    .width(Fill)
                    .align_x(Center),
                column![text_big("Transactions"), {
                    let element: Element<'a, Message> = if transactions.is_empty() {
                        center(text("No transactions yet")).into()
                    } else {
                        scrollable(
                            Column::from_iter(transactions.iter().map(|transaction| {
                                let block_height = transaction.block_height;
                                let txid = transaction.txid;
                                let txid_string = txid.to_string();
                                let event = transaction
                                    .events
                                    .iter()
                                    .find(|event| event.space.is_some());
                                let bumped = transaction
                                    .events
                                    .iter()
                                    .any(|event| event.kind == TxEventKind::FeeBump);

                                let tx_data_without_event = || -> Row<'a, Message> {
                                    let diff = transaction.received.to_sat() as i64
                                        - transaction.sent.to_sat() as i64;
                                    row![
                                        horizontal_space(),
                                        if diff >= 0 {
                                            text(format!("+{}", format_amount_number(diff as u64)))
                                                .style(move |theme: &Theme| text::Style {
                                                    color: Some(
                                                        theme
                                                            .extended_palette()
                                                            .success
                                                            .strong
                                                            .color,
                                                    ),
                                                })
                                        } else {
                                            text(format!("-{}", format_amount_number(-diff as u64)))
                                                .style(move |theme: &Theme| text::Style {
                                                    color: Some(
                                                        theme
                                                            .extended_palette()
                                                            .danger
                                                            .strong
                                                            .color,
                                                    ),
                                                })
                                        }
                                    ]
                                };

                                let tx_data_with_event =
                                    |action: &'static str,
                                     space: &'a str,
                                     amount: Option<Amount>|
                                     -> Row<'a, Message> {
                                        let slabel = SLabel::from_str(space).unwrap();
                                        row![
                                            text(action),
                                            button(text_monospace(space))
                                                .on_press(Message::SpacePress(slabel))
                                                .style(button::text)
                                                .padding(0),
                                            horizontal_space()
                                        ]
                                        .push_maybe(
                                            amount.map(|amount| text(format_amount(amount))),
                                        )
                                        .spacing(5)
                                        .align_y(Center)
                                    };

                                column![
                                    horizontal_rule(2.0),
                                    Space::with_height(10),
                                    row![
                                        container(
                                            button(
                                                Row::new()
                                                    .push_maybe(if bumped {
                                                        Some(text_icon(Icon::ArrowBigUpLines))
                                                    } else {
                                                        None
                                                    })
                                                    .push(text_monospace(format!(
                                                        "{} .. {}",
                                                        &txid_string[..8],
                                                        &txid_string[54..]
                                                    ))),
                                            )
                                            .style(button::text)
                                            .padding(0)
                                            .on_press(Message::TxidPress(txid))
                                        )
                                        .width(FillPortion(3)),
                                        match event {
                                            Some(TxEvent {
                                                kind: TxEventKind::Commit,
                                                space,
                                                ..
                                            }) => tx_data_with_event(
                                                "Commit",
                                                space.as_ref().unwrap(),
                                                None,
                                            ),
                                            Some(TxEvent {
                                                kind: TxEventKind::Open,
                                                space,
                                                details,
                                                ..
                                            }) => tx_data_with_event(
                                                "Open",
                                                space.as_ref().unwrap(),
                                                Some(
                                                    OpenEventDetails::deserialize(
                                                        details.as_ref().unwrap(),
                                                    )
                                                    .unwrap()
                                                    .initial_bid,
                                                ),
                                            ),
                                            Some(TxEvent {
                                                kind: TxEventKind::Bid,
                                                space,
                                                details,
                                                ..
                                            }) => tx_data_with_event(
                                                "Bid",
                                                space.as_ref().unwrap(),
                                                Some(
                                                    BidEventDetails::deserialize(
                                                        details.as_ref().unwrap(),
                                                    )
                                                    .unwrap()
                                                    .current_bid,
                                                ),
                                            ),
                                            Some(TxEvent {
                                                kind: TxEventKind::Transfer,
                                                space,
                                                ..
                                            }) => tx_data_with_event(
                                                "Transfer",
                                                space.as_ref().unwrap(),
                                                None
                                            ),
                                            Some(TxEvent {
                                                kind: TxEventKind::Renew,
                                                space,
                                                ..
                                            }) => tx_data_with_event(
                                                "Renew",
                                                space.as_ref().unwrap(),
                                                None
                                            ),
                                            Some(TxEvent {
                                                kind: TxEventKind::Buy,
                                                space,
                                                ..
                                            }) => tx_data_with_event(
                                                "Buy",
                                                space.as_ref().unwrap(),
                                                None
                                            ),
                                            _ => tx_data_without_event(),
                                        }
                                        .width(FillPortion(4)),
                                    ],
                                    match block_height {
                                        Some(block_height) => text_small(height_to_past_est(
                                            block_height,
                                            tip_height
                                        ),),
                                        None => text_small("Unconfirmed"),
                                    },
                                ]
                                .spacing(5)
                                .padding([10, 0])
                                .into()
                            }))
                            .push(Space::with_height(5))
                            .spacing(5),
                        )
                        .on_scroll(|viewport| {
                            Message::TxsListScrolled(
                                viewport.relative_offset().y,
                                transactions.len(),
                            )
                        })
                        .spacing(20)
                        .height(Fill)
                        .into()
                    };
                    element
                }]
                .padding([20, 20])
                .spacing(10)
                .height(Fill)
                .width(Fill),
            ]
            .height(Fill)
            .width(Fill)
            .into()
        }
    }
}
