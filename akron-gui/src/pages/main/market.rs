use crate::widget::base::{base_container, result_column};
use crate::widget::tx_result::{TxListMessage, TxResultWidget};
use crate::{
    helpers::*,
    widget::{
        form::Form,
        icon::{button_icon, Icon},
        tabs::TabsRow,
        text::{text_big, text_monospace},
    },
};
use iced::{
    widget::{column, container, row, text_editor},
    Border, Element, Fill, Theme,
};
use spaces_client::wallets::WalletResponse;
use spaces_wallet::bdk_wallet::serde_json;

#[derive(Debug, Default)]
pub struct BuyState {
    listing: text_editor::Content,
    fee_rate: String,
    error: Option<String>,
    tx_result: Option<TxResultWidget>,
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
    SLabelSelect(SLabel),
    PriceInput(String),
    BuySubmit,
    BuyResult(Result<WalletResponse, String>),
    SellSubmit,
    SellResult(Result<Listing, String>),
    CopyPress,
    TxResult(TxListMessage),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    Buy { listing: Listing },
    Sell { slabel: SLabel, price: Amount },
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
            Self::Buy(state) => {
                state.error = None;
                state.tx_result = None;
            }
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
                }
            }
            Message::BuyResult(Ok(w)) => {
                if w.result.iter().any(|r| r.error.is_some()) {
                    if let State::Buy(buy_state) = self {
                        buy_state.tx_result = Some(TxResultWidget::new(w));
                    }
                    return Action::None;
                }
                Action::ShowTransactions
            }
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
            Message::TxResult(msg) => {
                if let Self::Buy(state) = self {
                    if let Some(tx_result) = &mut state.tx_result {
                        tx_result.update(msg);
                    }
                }
                Action::None
            }
        }
    }

    pub fn view<'a>(&'a self, owned_spaces: &'a Vec<SLabel>) -> Element<'a, Message> {
        base_container(
            column![
                TabsRow::new()
                    .add_tab("Buy", matches!(self, Self::Buy(_)), Message::BuyTabPress,)
                    .add_tab("Sell", matches!(self, Self::Sell(_)), Message::SellTabPress,),
                match self {
                    Self::Buy(state) => {
                        column![
                            text_big("Buy space"),
                            result_column(
                                state.error.as_ref(),
                                state
                                    .tx_result
                                    .as_ref()
                                    .map(|tx| TxResultWidget::view(tx).map(Message::TxResult)),
                                [Form::new(
                                    "Buy",
                                    (listing_from_str(&state.listing.text()).is_some()
                                        && fee_rate_from_str(&state.fee_rate).is_some())
                                    .then_some(Message::BuySubmit)
                                )
                                .add_text_editor(
                                    "Listing",
                                    "JSON",
                                    &state.listing,
                                    Message::ListingAction
                                )
                                .into()]
                            )
                            .spacing(40),
                        ]
                        .spacing(40)
                    }
                    Self::Sell(state) => {
                        column![
                            text_big("Sell space"),
                            result_column(
                                state.error.as_ref(),
                                None,
                                [Form::new(
                                    "Generate Listing",
                                    (state.space.is_some()
                                        && amount_from_str(&state.price).is_some())
                                    .then_some(Message::SellSubmit),
                                )
                                .add_pick_list(
                                    "Space",
                                    owned_spaces.as_slice(),
                                    state.space.as_ref(),
                                    Message::SLabelSelect,
                                )
                                .add_text_input("Price", "sat", &state.price, Message::PriceInput,)
                                .into(),]
                            ),
                        ]
                        .push_maybe(state.listing.as_ref().map(|listing| {
                            container(row![
                                text_monospace(listing).width(Fill),
                                button_icon(Icon::Copy).on_press(Message::CopyPress)
                            ])
                            .padding(10)
                            .style(|theme: &Theme| {
                                let palette = theme.extended_palette();
                                container::Style::default()
                                    .background(palette.background.base.color)
                                    .border(Border {
                                        radius: 6.0.into(),
                                        width: 1.0,
                                        color: palette.background.strong.color,
                                    })
                            })
                        }))
                        .spacing(40)
                    }
                }
                .spacing(40)
            ]
            .spacing(40),
        )
        .into()
    }
}
