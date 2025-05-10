use crate::{
    helpers::*,
    widget::{
        form::Form,
        icon::{Icon, button_icon},
        tabs::TabsRow,
        text::{error_block, text_big, text_monospace},
    },
};
use iced::{
    Border, Element, Fill, Theme,
    widget::{column, container, row, text_editor},
};
use spaces_wallet::bdk_wallet::serde_json;

#[derive(Debug, Default)]
pub struct BuyState {
    listing: text_editor::Content,
    fee_rate: String,
    error: Option<String>,
}

#[derive(Debug, Default)]
pub struct SellState {
    space: Option<SLabel>,
    price: String,
    listing: Option<String>,
    error: Option<String>,
}

#[derive(Debug)]
pub enum State {
    Buy(BuyState),
    Sell(SellState),
}

impl Default for State {
    fn default() -> Self {
        Self::Buy(Default::default())
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    BuyTabPress,
    SellTabPress,
    ListingAction(text_editor::Action),
    FeeRateInput(String),
    SLabelSelect(SLabel),
    PriceInput(String),
    BuySubmit,
    BuyResult(Result<(), String>),
    SellSubmit,
    SellResult(Result<Listing, String>),
    CopyPress,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    Buy {
        listing: Listing,
        fee_rate: Option<FeeRate>,
    },
    Sell {
        slabel: SLabel,
        price: Amount,
    },
    WriteClipboard(String),
    ShowTransactions,
}

impl State {
    fn as_buy(&mut self) -> &mut BuyState {
        match self {
            Self::Buy(state) => state,
            _ => panic!("Expected Buy state"),
        }
    }

    fn as_sell(&mut self) -> &mut SellState {
        match self {
            Self::Sell(state) => state,
            _ => panic!("Expected Sell state"),
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match self {
            Self::Buy(state) => state.error = None,
            Self::Sell(state) => state.error = None,
        }
        match message {
            Message::BuyTabPress => {
                *self = Self::Buy(Default::default());
                Action::None
            }
            Message::SellTabPress => {
                *self = Self::Sell(Default::default());
                Action::None
            }
            Message::ListingAction(action) => {
                self.as_buy().listing.perform(action);
                Action::None
            }
            Message::FeeRateInput(fee_rate) => {
                if is_fee_rate_input(&fee_rate) {
                    self.as_buy().fee_rate = fee_rate
                }
                Action::None
            }
            Message::SLabelSelect(slabel) => {
                self.as_sell().space = Some(slabel);
                Action::None
            }
            Message::PriceInput(price) => {
                if is_amount_input(&price) {
                    self.as_sell().price = price;
                }
                Action::None
            }
            Message::BuySubmit => {
                let state = self.as_buy();
                Action::Buy {
                    listing: listing_from_str(&state.listing.text()).unwrap(),
                    fee_rate: fee_rate_from_str(&state.fee_rate).unwrap(),
                }
            }
            Message::BuyResult(Ok(())) => Action::ShowTransactions,
            Message::BuyResult(Err(err)) => {
                if let Self::Buy(state) = self {
                    state.error = Some(err);
                }
                Action::None
            }
            Message::SellSubmit => {
                let state = self.as_sell();
                Action::Sell {
                    slabel: state.space.clone().unwrap(),
                    price: amount_from_str(&state.price).unwrap(),
                }
            }
            Message::SellResult(Ok(value)) => {
                if let Self::Sell(state) = self {
                    state.listing = Some(serde_json::to_string_pretty(&value).unwrap());
                }
                Action::None
            }
            Message::SellResult(Err(err)) => {
                if let Self::Sell(state) = self {
                    state.error = Some(err);
                }
                Action::None
            }
            Message::CopyPress => Action::WriteClipboard(self.as_sell().listing.clone().unwrap()),
        }
    }

    pub fn view<'a>(&'a self, owned_spaces: &'a Vec<SLabel>) -> Element<'a, Message> {
        column![
            TabsRow::new()
                .add_tab("Buy", matches!(self, Self::Buy(_)), Message::BuyTabPress,)
                .add_tab("Sell", matches!(self, Self::Sell(_)), Message::SellTabPress,),
            match self {
                Self::Buy(state) => {
                    column![
                        text_big("Buy space"),
                        error_block(state.error.as_ref()),
                        Form::new(
                            "Buy",
                            (listing_from_str(&state.listing.text()).is_some()
                                && fee_rate_from_str(&state.fee_rate).is_some())
                            .then_some(Message::BuySubmit),
                        )
                        .add_text_editor("Listing", "JSON", &state.listing, Message::ListingAction)
                        .add_text_input(
                            "Fee rate",
                            "sat/vB (auto if empty)",
                            &state.fee_rate,
                            Message::FeeRateInput,
                        )
                    ]
                }
                Self::Sell(state) => {
                    column![
                        text_big("Sell space"),
                        error_block(state.error.as_ref()),
                        Form::new(
                            "Generate Listing",
                            (state.space.is_some() && amount_from_str(&state.price).is_some())
                                .then_some(Message::SellSubmit),
                        )
                        .add_pick_list(
                            "Space",
                            owned_spaces.as_slice(),
                            state.space.as_ref(),
                            Message::SLabelSelect,
                        )
                        .add_text_input(
                            "Price",
                            "sat",
                            &state.price,
                            Message::PriceInput,
                        ),
                    ]
                    .push_maybe(state.listing.as_ref().map(|listing| {
                        container(row![
                            text_monospace(listing).width(Fill),
                            button_icon(Icon::Copy).on_press(Message::CopyPress)
                        ])
                        .style(|theme: &Theme| {
                            let palette = theme.extended_palette();
                            container::Style::default()
                                .background(palette.background.base.color)
                                .border(Border {
                                    radius: 2.0.into(),
                                    width: 1.0,
                                    color: palette.background.strong.color,
                                })
                        })
                        .padding(10)
                    }))
                }
            }
            .spacing(10)
            .padding([60, 100])
        ]
        .padding([60, 0])
        .into()
    }
}
