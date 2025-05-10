use iced::{
    Center, Element, Fill, font,
    widget::{
        Column, Row, Space, button, center, column, container, horizontal_rule, row, scrollable,
        text,
    },
};

use super::state::SpacesCollection;
use crate::{
    client::*,
    helpers::*,
    widget::{
        form::{Form, text_input},
        icon::{Icon, button_icon, text_icon, text_input_icon},
        rect,
        tabs::TabsRow,
        text::{error_block, text_big, text_bold, text_monospace, text_monospace_bold, text_small},
    },
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    Owned,
    Bidding,
}

#[derive(Debug, Default)]
pub struct State {
    slabel: Option<SLabel>,
    search: String,
    filter: Filter,
    amount: String,
    fee_rate: String,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackPress,
    SLabelPress(SLabel),
    CopySLabelPress(SLabel),
    CopyOutpointPress(OutPoint),
    SearchInput(String),
    FilterPress(Filter),
    AmountInput(String),
    FeeRateInput(String),
    OpenSubmit,
    BidSubmit,
    RegisterSubmit,
    RenewSubmit,
    ClientResult(Result<(), String>),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    WriteClipboard(String),
    GetSpaceInfo {
        slabel: SLabel,
    },
    OpenSpace {
        slabel: SLabel,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    },
    BidSpace {
        slabel: SLabel,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    },
    RegisterSpace {
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    },
    RenewSpace {
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    },
    ShowTransactions,
}

impl State {
    pub fn reset_inputs(&mut self) {
        self.amount = Default::default();
        self.fee_rate = Default::default();
    }

    pub fn reset(&mut self) {
        self.reset_inputs();
        if self.slabel.is_some() {
            self.slabel = Default::default();
        } else {
            self.search = Default::default();
        }
    }

    pub fn set_slabel(&mut self, slabel: &SLabel) {
        self.reset_inputs();
        self.slabel = Some(slabel.clone())
    }

    pub fn get_slabel(&self) -> Option<SLabel> {
        self.slabel.clone()
    }

    pub fn update(&mut self, message: Message) -> Action {
        self.error = None;
        match message {
            Message::BackPress => {
                self.slabel = None;
                Action::None
            }
            Message::SLabelPress(slabel) => {
                self.slabel = Some(slabel.clone());
                Action::GetSpaceInfo { slabel }
            }
            Message::CopySLabelPress(slabel) => Action::WriteClipboard(slabel.to_string()),
            Message::CopyOutpointPress(outpoint) => Action::WriteClipboard(outpoint.to_string()),
            Message::SearchInput(search) => {
                if is_slabel_input(&search) {
                    self.search = search;
                    if let Some(slabel) = slabel_from_str(&self.search) {
                        Action::GetSpaceInfo { slabel }
                    } else {
                        Action::None
                    }
                } else {
                    Action::None
                }
            }
            Message::FilterPress(filter) => {
                self.filter = filter;
                Action::None
            }
            Message::AmountInput(amount) => {
                if is_amount_input(&amount) {
                    self.amount = amount
                }
                Action::None
            }
            Message::FeeRateInput(fee_rate) => {
                if is_fee_rate_input(&fee_rate) {
                    self.fee_rate = fee_rate
                }
                Action::None
            }
            Message::OpenSubmit => Action::OpenSpace {
                slabel: self.slabel.as_ref().unwrap().clone(),
                amount: amount_from_str(&self.amount).unwrap(),
                fee_rate: fee_rate_from_str(&self.fee_rate).unwrap(),
            },
            Message::BidSubmit => Action::BidSpace {
                slabel: self.slabel.as_ref().unwrap().clone(),
                amount: amount_from_str(&self.amount).unwrap(),
                fee_rate: fee_rate_from_str(&self.fee_rate).unwrap(),
            },
            Message::RegisterSubmit => Action::RegisterSpace {
                slabel: self.slabel.as_ref().unwrap().clone(),
                fee_rate: fee_rate_from_str(&self.fee_rate).unwrap(),
            },
            Message::RenewSubmit => Action::RenewSpace {
                slabel: self.slabel.as_ref().unwrap().clone(),
                fee_rate: fee_rate_from_str(&self.fee_rate).unwrap(),
            },
            Message::ClientResult(Ok(())) => {
                self.reset_inputs();
                Action::ShowTransactions
            }
            Message::ClientResult(Err(err)) => {
                self.error = Some(err);
                Action::None
            }
        }
    }

    fn open_form(&self) -> Element<'_, Message> {
        Form::new(
            "Open",
            (amount_from_str(&self.amount).is_some()
                && fee_rate_from_str(&self.fee_rate).is_some())
            .then_some(Message::OpenSubmit),
        )
        .add_text_input("Amount", "sat", &self.amount, Message::AmountInput)
        .add_text_input(
            "Fee rate",
            "sat/vB (auto if empty)",
            &self.fee_rate,
            Message::FeeRateInput,
        )
        .into()
    }

    fn bid_form(&self, current_bid: Amount) -> Element<'_, Message> {
        Form::new(
            "Bid",
            (amount_from_str(&self.amount).is_some_and(|amount| amount > current_bid)
                && fee_rate_from_str(&self.fee_rate).is_some())
            .then_some(Message::BidSubmit),
        )
        .add_text_input("Amount", "sat", &self.amount, Message::AmountInput)
        .add_text_input(
            "Fee rate",
            "sat/vB (auto if empty)",
            &self.fee_rate,
            Message::FeeRateInput,
        )
        .into()
    }

    fn register_form(&self) -> Element<'_, Message> {
        Form::new(
            "Register",
            fee_rate_from_str(&self.fee_rate).map(|_| Message::RegisterSubmit),
        )
        .add_text_input(
            "Fee rate",
            "sat/vB (auto if empty)",
            &self.fee_rate,
            Message::FeeRateInput,
        )
        .into()
    }

    fn renew_form(&self) -> Element<'_, Message> {
        Form::new(
            "Renew",
            fee_rate_from_str(&self.fee_rate).map(|_| Message::RenewSubmit),
        )
        .add_text_input(
            "Fee rate",
            "sat/vB (auto if empty)",
            &self.fee_rate,
            Message::FeeRateInput,
        )
        .into()
    }

    fn open_view(&self) -> Element<'_, Message> {
        row![
            timeline::view(0, "Make an open to propose the space for auction"),
            column![
                text_big("Open space"),
                error_block(self.error.as_ref()),
                self.open_form(),
            ]
            .spacing(10),
        ]
        .into()
    }

    fn bid_view(
        &self,
        tip_height: u32,
        claim_height: Option<u32>,
        current_bid: Amount,
        is_winning: bool,
    ) -> Element<'_, Message> {
        row![
            timeline::view(
                if claim_height.is_none() { 1 } else { 2 },
                claim_height.map_or(
                    "Make a bid to improve the chance of moving the space to auction".to_string(),
                    |height| format!("Auction ends {}", height_to_future_est(height, tip_height))
                )
            ),
            column![
                text_big("Bid space"),
                error_block(self.error.as_ref()),
                row![
                    text("Current bid").size(14),
                    text_bold(format_amount(current_bid).to_string()).size(14),
                ]
                .spacing(5),
                row![
                    text("Winning bidder").size(14),
                    text_bold(if is_winning { "you" } else { "not you" }).size(14),
                ]
                .spacing(5),
                self.bid_form(current_bid),
            ]
            .spacing(10),
        ]
        .into()
    }

    fn register_view(&self, current_bid: Amount, is_winning: bool) -> Element<'_, Message> {
        row![
            timeline::view(
                3,
                if is_winning {
                    "You can register the space"
                } else {
                    "The auction is ended, but you still can outbid"
                }
            ),
            if is_winning {
                column![
                    text_big("Register space"),
                    error_block(self.error.as_ref()),
                    self.register_form(),
                ]
                .spacing(10)
            } else {
                column![
                    text_big("Bid space"),
                    error_block(self.error.as_ref()),
                    row![
                        text("Current bid").size(14),
                        text_bold(format_amount(current_bid)).size(14),
                    ]
                    .spacing(5),
                    self.bid_form(current_bid),
                ]
                .spacing(10)
            }
        ]
        .into()
    }

    fn registered_view<'a>(
        &'a self,
        tip_height: u32,
        expire_height: u32,
        outpoint: &'a OutPoint,
        is_owned: bool,
    ) -> Element<'a, Message> {
        row![
            column![
                text(format!(
                    "Expires {}",
                    height_to_future_est(expire_height, tip_height)
                )),
                row![
                    text("Outpoint"),
                    text_monospace({
                        let txid_string = outpoint.txid.to_string();
                        format!(
                            "{}..{}:{}",
                            &txid_string[..8],
                            &txid_string[54..],
                            outpoint.vout,
                        )
                    }),
                    button_icon(Icon::Copy)
                        .style(button::text)
                        .on_press(Message::CopyOutpointPress(*outpoint)),
                ]
                .spacing(5)
                .align_y(Center)
            ]
            .spacing(5)
            .width(Fill),
            if is_owned {
                column![
                    text_big("Renew space"),
                    error_block(self.error.as_ref()),
                    self.renew_form(),
                ]
                .spacing(10)
            } else {
                column![]
            }
            .width(Fill)
        ]
        .into()
    }

    pub fn view<'a>(
        &'a self,
        tip_height: u32,
        spaces: &'a SpacesCollection,
        winning_spaces: &'a [SLabel],
        outbid_spaces: &'a [SLabel],
        owned_spaces: &'a [SLabel],
    ) -> Element<'a, Message> {
        if let Some(slabel) = self.slabel.as_ref() {
            let covenant = spaces.get_covenant(slabel);
            column![
                row![
                    button(text_icon(Icon::ChevronLeft).size(20))
                        .style(button::text)
                        .on_press(Message::BackPress),
                    text_monospace_bold(slabel.to_string()).size(20),
                    button_icon(Icon::Copy)
                        .style(button::text)
                        .on_press(Message::CopySLabelPress(slabel.clone())),
                ]
                .spacing(5)
                .align_y(Center),
                horizontal_rule(3),
                match covenant {
                    None => center(text("Loading")).into(),
                    Some(None) => self.open_view(),
                    Some(Some(Covenant::Bid {
                        claim_height,
                        total_burned,
                        ..
                    })) => {
                        let is_winning = winning_spaces.contains(slabel);
                        if claim_height.is_some_and(|height| height <= tip_height) {
                            self.register_view(*total_burned, is_winning)
                        } else {
                            self.bid_view(tip_height, *claim_height, *total_burned, is_winning)
                        }
                    }
                    Some(Some(Covenant::Transfer { expire_height, .. })) => {
                        let is_owned = owned_spaces.contains(slabel);
                        self.registered_view(
                            tip_height,
                            *expire_height,
                            spaces.get_outpoint(slabel).unwrap(),
                            is_owned,
                        )
                    }
                    Some(Some(Covenant::Reserved)) => center(text("The space is locked")).into(),
                },
            ]
            .padding(20)
            .spacing(20)
        } else {
            let mut slabels: Vec<&SLabel> = if self.search.is_empty() {
                match self.filter {
                    Filter::Owned => owned_spaces.iter().collect(),
                    Filter::Bidding => winning_spaces.iter().chain(outbid_spaces).collect(),
                }
            } else {
                owned_spaces
                    .iter()
                    .chain(winning_spaces.iter())
                    .chain(outbid_spaces.iter())
                    .filter(|s| s.as_str_unprefixed().unwrap().contains(&self.search))
                    .collect()
            };
            slabels.sort_unstable_by_key(|s| s.as_str_unprefixed().unwrap());

            let card = |slabel: &SLabel| -> Element<'a, Message> {
                enum State {
                    None,
                    Success,
                    Danger,
                }

                let (data, state): (Element<'a, Message>, State) = match spaces.get_covenant(slabel)
                {
                    None => (Space::with_width(Fill).into(), State::None),
                    Some(None) => (text_small("Available").width(Fill).into(), State::None),
                    Some(Some(Covenant::Bid {
                        claim_height,
                        total_burned,
                        ..
                    })) => {
                        let is_claimable = claim_height.is_some_and(|height| height <= tip_height);
                        let is_winning = winning_spaces.contains(slabel);
                        (
                            column![
                                text_small("In auction"),
                                text_small(format!(
                                    "Highest bid: {} ({})",
                                    format_amount(*total_burned),
                                    if is_winning { "you" } else { "not you" }
                                )),
                                if is_claimable {
                                    text_small("Can be claimed")
                                } else if let Some(claim_height) = claim_height {
                                    text_small(format!(
                                        "Ends {}",
                                        height_to_future_est(*claim_height, tip_height)
                                    ))
                                } else {
                                    text_small("Pre-auction")
                                }
                            ]
                            .width(Fill)
                            .into(),
                            if is_winning {
                                if is_claimable {
                                    State::Success
                                } else {
                                    State::None
                                }
                            } else {
                                State::Danger
                            },
                        )
                    }
                    Some(Some(Covenant::Transfer { expire_height, .. })) => {
                        let is_owned = owned_spaces.contains(slabel);
                        (
                            column![
                                text_small(if is_owned { "Owned" } else { "Registered" }),
                                text_small(format!(
                                    "Expires {}",
                                    height_to_future_est(*expire_height, tip_height)
                                )),
                            ]
                            .width(Fill)
                            .into(),
                            if is_owned && *expire_height <= tip_height {
                                State::Danger
                            } else {
                                State::None
                            },
                        )
                    }
                    Some(Some(Covenant::Reserved)) => {
                        (text_small("Reserved").width(Fill).into(), State::None)
                    }
                };
                column![
                    horizontal_rule(2.0),
                    Space::with_height(10),
                    row![
                        button(
                            Row::new()
                                .push_maybe(match state {
                                    State::None => None,
                                    State::Success => Some(rect::Rect::new(15.0, 15.0).style(
                                        |theme: &iced::Theme| {
                                            rect::Style {
                                                border: iced::Border {
                                                    radius: 3.into(),
                                                    ..Default::default()
                                                },
                                                background: Some(theme.palette().success.into()),
                                                inner: None,
                                            }
                                        }
                                    )),
                                    State::Danger => Some(rect::Rect::new(15.0, 15.0).style(
                                        |theme: &iced::Theme| {
                                            rect::Style {
                                                border: iced::Border {
                                                    radius: 3.into(),
                                                    ..Default::default()
                                                },
                                                background: Some(theme.palette().danger.into()),
                                                inner: None,
                                            }
                                        }
                                    )),
                                })
                                .push(text_monospace(slabel.to_string()))
                                .spacing(5)
                                .align_y(Center)
                        )
                        .style(button::text)
                        .width(Fill)
                        .on_press(Message::SLabelPress(slabel.clone())),
                        data,
                    ]
                    .align_y(Center)
                    .spacing(5),
                ]
                .spacing(5)
                .padding([10, 0])
                .into()
            };

            column![
                Column::new()
                    .push(
                        container(
                            text_input("space", &self.search)
                                .icon(text_input_icon(Icon::At, None, 10.0))
                                .on_input(Message::SearchInput)
                                .font(font::Font::MONOSPACE)
                                .size(20)
                                .padding([10, 20]),
                        )
                        .padding([30, 100]),
                    )
                    .push_maybe(if self.search.is_empty() {
                        Some(
                            TabsRow::new()
                                .add_tab(
                                    "Owned",
                                    self.filter == Filter::Owned,
                                    Message::FilterPress(Filter::Owned),
                                )
                                .add_tab(
                                    "Bidding",
                                    self.filter == Filter::Bidding,
                                    Message::FilterPress(Filter::Bidding),
                                ),
                        )
                    } else {
                        None
                    }),
                scrollable(
                    Column::new()
                        .push_maybe(
                            slabel_from_str(&self.search)
                                .filter(|slabel| !slabels.contains(&slabel))
                                .map(|slabel| card(&slabel)),
                        )
                        .extend(slabels.into_iter().map(card))
                        .push(Space::with_height(5))
                        .spacing(5),
                )
                .spacing(10)
                .height(Fill)
                .width(Fill),
            ]
            .padding([20, 20])
            .spacing(50)
        }
        .into()
    }
}

mod timeline {
    use crate::widget::rect::*;
    use iced::{
        Border, Center, Element, Fill, Theme,
        widget::{Column, Row, text},
    };

    const CIRCLE_RADIUS: f32 = 20.0;
    const LINE_WIDTH: f32 = 3.0;
    const LINE_HEIGHT: f32 = 50.0;
    const ROW_SPACING: f32 = 10.0;

    fn circle<'a>(filled: bool, border: bool, inner: bool) -> Rect<'a> {
        Rect::new(CIRCLE_RADIUS * 2.0, CIRCLE_RADIUS * 2.0).style(move |theme: &Theme| {
            let palette = theme.palette();
            Style {
                border: Border {
                    color: if border {
                        palette.primary
                    } else {
                        palette.text
                    },
                    width: LINE_WIDTH,
                    radius: CIRCLE_RADIUS.into(),
                },
                background: if filled {
                    Some(palette.primary.into())
                } else {
                    None
                },
                inner: if inner {
                    Some(Inner {
                        border: Border {
                            radius: CIRCLE_RADIUS.into(),
                            ..Border::default()
                        },
                        background: Some(palette.primary.into()),
                        padding: (CIRCLE_RADIUS / 2.0).into(),
                    })
                } else {
                    None
                },
            }
        })
    }

    fn line<'a>(filled: bool) -> Rect<'a> {
        Rect::new(CIRCLE_RADIUS * 2.0, LINE_HEIGHT).style(move |theme: &Theme| {
            let palette = theme.palette();
            Style {
                inner: Some(Inner {
                    background: Some(
                        if filled {
                            palette.primary
                        } else {
                            palette.text
                        }
                        .into(),
                    ),
                    padding: [0.0, CIRCLE_RADIUS - LINE_WIDTH / 2.0].into(),
                    ..Inner::default()
                }),
                ..Style::default()
            }
        })
    }

    fn space<'a>() -> Rect<'a> {
        Rect::new(CIRCLE_RADIUS * 2.0, LINE_HEIGHT)
    }

    pub fn view<'a, Message: 'a>(
        state: u8,
        label: impl text::IntoFragment<'a> + Clone,
    ) -> Element<'a, Message> {
        const LABELS: [&str; 4] = ["Open", "Pre-auction", "Auction", "Claim"];
        if state > LABELS.len() as u8 {
            panic!("state is out of range");
        }
        Column::from_iter((0..(LABELS.len() as u8) * 2).map(|i| {
            let c = i % 2 == 0;
            let n = i / 2;
            let o = n.cmp(&state);
            let row = Row::new()
                .push(if c {
                    circle(o.is_lt(), o.is_le(), o.is_eq())
                } else if n == LABELS.len() as u8 - 1 {
                    space()
                } else {
                    line(o.is_lt())
                })
                .push_maybe(if c {
                    Some(text(LABELS[n as usize]))
                } else if (state == LABELS.len() as u8 && state - n == 1) || o.is_eq() {
                    Some(text(label.clone()))
                } else {
                    None
                })
                .spacing(ROW_SPACING);
            if c { row.align_y(Center) } else { row }.into()
        }))
        .width(Fill)
        .into()
    }
}
