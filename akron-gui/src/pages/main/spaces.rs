use super::state::SpacesCollection;
use crate::widget::base::{base_container, result_column};
use crate::widget::form::STANDARD_PADDING;
use crate::widget::text::text_semibold;
use crate::widget::tx_result::{TxListMessage, TxResultWidget};
use crate::{
    client::*,
    helpers::*,
    widget::{
        form::Form,
        icon::{button_icon, text_icon, text_input_icon, Icon},
        rect,
        tabs::TabsRow,
        text::{error_block, text_big, text_bold, text_monospace, text_small},
    },
};
use iced::border::rounded;
use iced::widget::text_input;
use iced::{
    font,
    widget::{
        button, center, column, container, horizontal_rule, row, scrollable, text, Column, Row,
        Space,
    },
    Center, Color, Element, Fill, Font, Theme,
};
use spaces_protocol::bitcoin::XOnlyPublicKey;

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
    error: Option<String>,
    tx_result: Option<TxResultWidget>,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackPress,
    SLabelPress(SLabel),
    CopySLabelPress(SLabel),
    CopyOutpointPress(OutPoint),
    CopyPublicKeyPress(XOnlyPublicKey),
    SearchInput(String),
    FilterPress(Filter),
    AmountInput(String),
    OpenSubmit,
    BidSubmit,
    RegisterSubmit,
    RenewSubmit,
    ClientResult(Result<WalletResponse, String>),
    TxResult(TxListMessage),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    WriteClipboard(String),
    GetSpaceInfo { slabel: SLabel },
    OpenSpace { slabel: SLabel, amount: Amount },
    BidSpace { slabel: SLabel, amount: Amount },
    RegisterSpace { slabel: SLabel },
    RenewSpace { slabel: SLabel },
    ShowTransactions,
}

impl State {
    pub fn reset_inputs(&mut self) {
        self.amount = Default::default();
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
        self.tx_result = None;

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
            Message::CopyPublicKeyPress(pubkey) => Action::WriteClipboard(pubkey.to_string()),
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
            Message::OpenSubmit => Action::OpenSpace {
                slabel: self.slabel.as_ref().unwrap().clone(),
                // TODO: allow users to choose during open but don't encourage them
                // must be set under a check box e.g. advanced options ...etc
                amount: amount_from_str("1000").unwrap(),
            },
            Message::BidSubmit => Action::BidSpace {
                slabel: self.slabel.as_ref().unwrap().clone(),
                amount: amount_from_str(&self.amount).unwrap(),
            },
            Message::RegisterSubmit => Action::RegisterSpace {
                slabel: self.slabel.as_ref().unwrap().clone(),
            },
            Message::RenewSubmit => Action::RenewSpace {
                slabel: self.slabel.as_ref().unwrap().clone(),
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

    fn open_form(&self) -> Element<'_, Message> {
        Form::new("Start auction", Some(Message::OpenSubmit)).into()
    }

    fn bid_form(&self, current_bid: Amount) -> Element<'_, Message> {
        Form::new(
            "Bid",
            (amount_from_str(&self.amount).is_some_and(|amount| amount > current_bid))
                .then_some(Message::BidSubmit),
        )
        .add_text_input("Amount", "sat", &self.amount, Message::AmountInput)
        .into()
    }

    fn register_form(&self) -> Element<'_, Message> {
        Form::new("Register", Some(Message::RegisterSubmit)).into()
    }

    fn renew_form(&self) -> Element<'_, Message> {
        Form::new("Renew", Some(Message::RenewSubmit)).into()
    }

    fn open_view(&self) -> Element<'_, Message> {
        timeline_container(
            0,
            "Click 'Start Auction' to begin.",
            result_column(
                self.error.as_ref(),
                self.tx_result
                    .as_ref()
                    .map(|tx| TxResultWidget::view(tx).map(Message::TxResult)),
                [self.open_form()],
            )
            .spacing(40),
        )
        .into()
    }

    fn bid_view(
        &self,
        tip_height: u32,
        claim_height: Option<u32>,
        current_bid: Amount,
        is_winning: bool,
    ) -> Element<'_, Message> {
        timeline_container(
            if claim_height.is_none() { 1 } else { 2 },
            claim_height.map_or(
                "Place a high bid to advance this space to auctions".to_string(),
                |height| format!("Auction ends {}", height_to_future_est(height, tip_height)),
            ),
            result_column(
                self.error.as_ref(),
                self.tx_result
                    .as_ref()
                    .map(|tx| TxResultWidget::view(tx).map(Message::TxResult)),
                [
                    column![
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
                    ]
                    .into(),
                    self.bid_form(current_bid),
                ],
            )
            .spacing(40),
        )
        .into()
    }

    fn register_view(&self, current_bid: Amount, is_winning: bool) -> Element<'_, Message> {
        timeline_container(
            3,
            if is_winning {
                "Congrats! Register the space before you get outbid."
            } else {
                "Space pending claim by winner. Outbid now to extend the auction."
            },
            if is_winning {
                result_column(
                    self.error.as_ref(),
                    self.tx_result
                        .as_ref()
                        .map(|tx| TxResultWidget::view(tx).map(Message::TxResult)),
                    [self.register_form()],
                )
                .spacing(10)
            } else {
                result_column(
                    self.error.as_ref(),
                    self.tx_result
                        .as_ref()
                        .map(|tx| TxResultWidget::view(tx).map(Message::TxResult)),
                    [
                        row![
                            text("Current bid").size(14),
                            text_bold(format_amount(current_bid)).size(14),
                        ]
                        .spacing(5)
                        .into(),
                        self.bid_form(current_bid),
                    ],
                )
                .spacing(10)
            },
        )
        .into()
    }

    fn registered_view<'a>(
        &'a self,
        space: &SLabel,
        tip_height: u32,
        expire_height: u32,
        owner: (&'a OutPoint, &'a Option<XOnlyPublicKey>),
        is_owned: bool,
    ) -> Element<'a, Message> {
        let (outpoint, pubkey) = owner;
        base_container(
            column![
                container(
                    column![
                        // Pubkey heading
                        row![text_monospace(space.to_string())
                            .color(Color::BLACK)
                            .size(24),]
                        .push_maybe(if let Some(pubkey) = pubkey.as_ref() {
                            Some(
                                button_icon(Icon::Copy)
                                    .style(|t: &Theme, s: button::Status| {
                                        let p = t.extended_palette();
                                        let mut style = button::text(t, s);
                                        if matches!(s, button::Status::Active) {
                                            style.text_color = p.success.strong.color;
                                        }
                                        style
                                    })
                                    .on_press(Message::CopyPublicKeyPress(pubkey.clone())),
                            )
                        } else {
                            None
                        })
                        .push(Space::with_width(Fill))
                        .push(text_icon(Icon::Bitcoin).color(Color::BLACK).size(28))
                        .align_y(Center),
                        column![
                            // Pubkey
                            if let Some(pubkey) = pubkey.as_ref() {
                                let key = pubkey.to_string();
                                column![
                                    text_monospace(
                                        format!(
                                            "{}",
                                            &key[..32]
                                                .chars()
                                                .collect::<Vec<char>>()
                                                .chunks(4)
                                                .map(|chunk| chunk.iter().collect::<String>())
                                                .collect::<Vec<String>>()
                                                .join(" ")
                                        )
                                        .to_uppercase()
                                    )
                                    .style(text::success)
                                    .size(20),
                                    text_monospace(
                                        format!(
                                            "{}",
                                            &key[32..]
                                                .chars()
                                                .collect::<Vec<char>>()
                                                .chunks(4)
                                                .map(|chunk| chunk.iter().collect::<String>())
                                                .collect::<Vec<String>>()
                                                .join(" ")
                                        )
                                        .to_uppercase()
                                    )
                                    .style(text::success)
                                    .size(20),
                                ]
                                .spacing(5)
                                .width(Fill)
                                .align_x(Center)
                            } else {
                                column![text_monospace("<address not supported>")]
                                    .width(Fill)
                                    .align_x(Center)
                            }
                        ]
                        .spacing(10)
                        .width(Fill),
                        column![
                            row![
                                text("Outpoint"),
                                Space::with_width(Fill),
                                text_monospace({
                                    let txid_string = outpoint.txid.to_string();
                                    format!(
                                        "{}..{}:{}",
                                        &txid_string[..20],
                                        &txid_string[50..],
                                        outpoint.vout,
                                    )
                                }),
                                button_icon(Icon::Copy)
                                    .style(button::text)
                                    .on_press(Message::CopyOutpointPress(*outpoint)),
                            ]
                            .width(Fill)
                            .align_y(Center),
                            row![
                                text("Expires"),
                                Space::with_width(Fill),
                                text_bold(height_to_future_est(expire_height, tip_height))
                            ]
                            .width(Fill),
                        ]
                        .spacing(5)
                        .width(Fill)
                    ]
                    .spacing(40)
                )
                .width(Fill)
                .style(|_t: &Theme| {
                    container::Style {
                        border: rounded(8).color(Color::BLACK).width(1),
                        ..container::Style::default()
                    }
                })
                .padding(40),
                if is_owned {
                    column![
                        text_big("Actions"),
                        error_block(self.error.as_ref()),
                        if let Some(tx_widget) = &self.tx_result {
                            tx_widget.view().map(Message::TxResult)
                        } else {
                            text("").into()
                        },
                        self.renew_form(),
                    ]
                    .spacing(10)
                } else {
                    column![]
                }
                .width(Fill)
            ]
            .spacing(80),
        )
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
            container(
                column![
                    row![
                        button(text_icon(Icon::ChevronLeft).size(20))
                            .style(button::text)
                            .on_press(Message::BackPress),
                        text_semibold(slabel.to_string()).size(20),
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
                                slabel,
                                tip_height,
                                *expire_height,
                                spaces.get_outpoint(slabel).unwrap(),
                                is_owned,
                            )
                        }
                        Some(Some(Covenant::Reserved)) =>
                            center(text("The space is locked")).into(),
                    },
                ]
                .padding([20, 0])
                .spacing(20),
            )
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
                container(
                    column![row![
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
                                .push(text_semibold(slabel.to_string()).size(20))
                                .spacing(5)
                                .align_y(Center)
                        )
                        .style(button::text)
                        .width(Fill)
                        .on_press(Message::SLabelPress(slabel.clone())),
                        data,
                    ]
                    .align_y(Center)
                    .spacing(20),]
                    .spacing(20),
                )
                .style(|_t: &Theme| container::Style {
                    background: Some(Color::from_rgb8(0xFC, 0xFD, 0xFE).into()),
                    border: rounded(8)
                        .width(1)
                        .color(Color::from_rgb8(0xDD, 0xE3, 0xEA)),
                    ..container::Style::default()
                })
                .padding(STANDARD_PADDING)
                .into()
            };

            container(scrollable(
                container(
                    column![
                        Column::new()
                            .push(
                                container(
                                    text_input("Enter a space name ...", &self.search)
                                        .icon(text_input_icon(Icon::AtSign, Some(24.into()), 10.0))
                                        .on_input(Message::SearchInput)
                                        .font(Font {
                                            weight: font::Weight::Semibold,
                                            family: font::Family::Name("Karla"),
                                            ..font::Font::DEFAULT
                                        })
                                        .style(|theme: &Theme, status: text_input::Status| {
                                            let p = theme.extended_palette();
                                            let mut style = text_input::default(theme, status);
                                            style.border = style.border.rounded(8);
                                            match status {
                                                text_input::Status::Active => {
                                                    style.icon = p.primary.weak.color
                                                }
                                                _ => style.icon = p.primary.base.color,
                                            };
                                            style
                                        })
                                        .size(18)
                                        .width(600)
                                        .padding([24, 24]),
                                )
                                .width(Fill)
                                .align_x(Center)
                                .align_y(Center)
                                .padding([65, 100]),
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
                        Column::new()
                            .push_maybe(if slabels.is_empty() && self.search.is_empty() {
                                column![
                                    horizontal_rule(2),
                                    container(
                                        container(
                                            text(format!(
                                                "No {}",
                                                match &self.filter {
                                                    Filter::Owned => "owned spaces",
                                                    Filter::Bidding => "bids",
                                                }
                                            ))
                                            .size(16)
                                        )
                                        .align_x(Center)
                                        .padding(80)
                                        .width(Fill)
                                    )
                                    .align_x(Center)
                                    .width(Fill)
                                ]
                                .spacing(40)
                                .width(Fill)
                                .into()
                            } else {
                                None
                            })
                            .push_maybe(
                                slabel_from_str(&self.search)
                                    .filter(|slabel| !slabels.contains(&slabel))
                                    .map(|slabel| card(&slabel)),
                            )
                            .extend(slabels.into_iter().map(card))
                            .push(Space::with_height(5))
                            .spacing(10),
                    ]
                    .width(800)
                    .padding([20, 20])
                    .spacing(50),
                )
                .width(Fill)
                .align_x(Center),
            ))
            .width(Fill)
            .height(Fill)
        }
        .into()
    }
}

// same as base container but has a timeline at the top
fn timeline_container<'a, Message: 'a>(
    step: u8,
    desc: impl text::IntoFragment<'a>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    base_container(
        column![timeline_bar(step, desc), content.into()]
            .spacing(40)
            .align_x(Center),
    )
}

fn timeline_bar<'a, Message: 'a>(
    step: u8,
    description: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
    column!(
        timeline_widget::view(step),
        text_semibold(description).size(20),
    )
    .align_x(Center)
    .spacing(40)
    .into()
}

mod timeline_widget {
    use crate::widget::text::text_bold;
    use iced::widget::container;
    use iced::{
        border,
        widget::{text, Column, Row},
        Center, Element, Padding, Theme, Top,
    };

    const LINE_LENGTH: f32 = 60.0;

    pub fn view<'a, Message: 'a>(state: u8) -> Element<'a, Message> {
        const LABELS: [&str; 4] = ["Open", "Pre-auction", "Auction", "Claim"];
        if state > LABELS.len() as u8 {
            panic!("state is out of range");
        }

        let mut timeline_row = Row::new();
        for n in 0..LABELS.len() as u8 {
            let o = n.cmp(&state);
            let step_column = Row::new()
                .push(
                    Column::new()
                        .spacing(14)
                        .push(
                            container(text_bold(format!("{}", n + 1)).size(14))
                                .width(36)
                                .height(36)
                                .align_y(Center)
                                .align_x(Center)
                                .style(move |theme: &Theme| {
                                    let palette = theme.extended_palette();
                                    container::Style {
                                        background: if o.is_eq() {
                                            Some(palette.primary.weak.color.into())
                                        } else {
                                            Some(palette.background.weak.color.into())
                                        },
                                        border: border::rounded(8),
                                        ..container::Style::default()
                                    }
                                }),
                        )
                        .push(
                            text_bold(LABELS[n as usize])
                                .width(100)
                                .size(14)
                                .align_x(Center)
                                .style(move |theme: &Theme| {
                                    let palette = theme.extended_palette();
                                    text::Style {
                                        color: if o.is_eq() {
                                            palette.primary.strong.color.into()
                                        } else {
                                            None
                                        },
                                        ..text::Style::default()
                                    }
                                }),
                        )
                        .align_x(Center),
                )
                .push(if n < LABELS.len() as u8 - 1 {
                    container(container("").width(LINE_LENGTH).height(1.2).style(
                        |theme: &Theme| {
                            let palette = theme.extended_palette();
                            container::Style {
                                background: Some(palette.background.weak.color.into()),
                                ..container::Style::default()
                            }
                        },
                    ))
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .into()
                } else {
                    container("")
                })
                .align_y(Top);
            timeline_row = timeline_row.push(step_column).align_y(Center);
        }

        timeline_row.align_y(Center).into()
    }
}
