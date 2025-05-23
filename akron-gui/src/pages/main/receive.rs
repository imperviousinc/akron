use super::state::AddressData;
use crate::widget::base::base_container;
use crate::widget::form::STANDARD_PADDING;
use crate::{
    client::*,
    widget::{
        icon::{button_icon, Icon},
        tabs::TabsRow,
        text::{text_big, text_monospace},
    },
};
use iced::{
    widget::{column, container, qr_code, row, text},
    Border, Center, Element, Fill, Theme,
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

        base_container(
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
                column![
                text_big(match self.0 {
                    AddressKind::Coin => "Coins-only address",
                    AddressKind::Space => "Spaces address",
                }),
                text(match self.0 {
                    AddressKind::Coin => "Bitcoin address suitable for receiving coins compatible with most bitcoin wallets.",
                    AddressKind::Space => "Bitcoin address suitable for receiving spaces and coins (Spaces compatible bitcoin wallets only).",
                })].spacing(10),
                column![
                    container(
                        row![
                            text_monospace(address.as_str()).size(12).width(Fill),
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
                    .padding(STANDARD_PADDING),

                ]
                .align_x(Center),
                container(qr_code(address.as_qr_code()).cell_size(7)).align_x(Center).width(Fill),
            ].width(Fill)
            .spacing(40)
        })).width(Fill).spacing(40)
        )
        .into()
    }
}
