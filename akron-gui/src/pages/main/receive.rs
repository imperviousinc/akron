use iced::{
    Border, Center, Element, Fill, Theme,
    widget::{column, container, qr_code, row, text},
};

use super::state::AddressData;
use crate::{
    client::*,
    widget::{
        icon::{Icon, button_icon},
        tabs::TabsRow,
        text::{text_big, text_monospace},
    },
};

#[derive(Debug)]
pub struct State(AddressKind);

impl Default for State {
    fn default() -> Self {
        Self(AddressKind::Coin)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TabPress(AddressKind),
    CopyPress(String),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    WriteClipboard(String),
}

impl State {
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::TabPress(address_kind) => {
                self.0 = address_kind;
                Action::None
            }
            Message::CopyPress(s) => Action::WriteClipboard(s),
        }
    }

    pub fn view<'a>(
        &self,
        coin_address: Option<&'a AddressData>,
        space_address: Option<&'a AddressData>,
    ) -> Element<'a, Message> {
        let address = match self.0 {
            AddressKind::Coin => coin_address,
            AddressKind::Space => space_address,
        };

        column![TabsRow::new()
            .add_tab(
                "Coins",
                matches!(self.0, AddressKind::Coin),
                Message::TabPress(AddressKind::Coin)
            )
            .add_tab(
                "Spaces",
                matches!(self.0, AddressKind::Space),
                Message::TabPress(AddressKind::Space)
            )]
        .push_maybe(address.map(|address| {
            column![
                text_big(match self.0 {
                    AddressKind::Coin => "Coins-only address",
                    AddressKind::Space => "Spaces address",
                }),
                text(match self.0 {
                    AddressKind::Coin => "Bitcoin address suitable for receiving coins compatible with most bitcoin wallets",
                    AddressKind::Space => "Bitcoin address suitable for receiving spaces and coins (Spaces compatible bitcoin wallets only)",
                }),
                column![
                    container(
                        row![
                            text_monospace(address.as_str()).width(Fill),
                            button_icon(Icon::Copy)
                                .on_press(Message::CopyPress(address.as_str().to_owned())),
                        ]
                        .align_y(Center)
                        .spacing(5)
                    )
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        container::Style::default()
                            .background(palette.background.base.color)
                            .border(Border {
                                radius: 7.0.into(),
                                width: 1.0,
                                color: palette.background.strong.color,
                            })
                    })
                    .padding(10),
                    qr_code(address.as_qr_code()).cell_size(7),
                ]
                .align_x(Center),
            ]
            .spacing(10)
            .padding([60, 100])
        }))
        .padding([60, 0])
        .into()
    }
}
