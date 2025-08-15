use iced::border::rounded;
use iced::widget::{container, text, Column, Container, Row};
use iced::{Alignment, Color, Element, Length, Theme};
use spaces_client::wallets::{TxResponse, WalletResponse};

#[derive(Debug, Clone)]
pub struct TxResultWidget {
    transactions: Vec<TxResponse>,
}

#[derive(Debug, Clone)]
pub enum TxListMessage {
    // TODO: if any interactivity needed later
}

impl TxResultWidget {
    pub fn new(response: WalletResponse) -> Self {
        TxResultWidget {
            transactions: response.result,
        }
    }

    pub fn update(&mut self, _message: TxListMessage) {
        // No state changes needed
    }

    pub fn view(&self) -> Element<'_, TxListMessage> {
        let content = if self.transactions.is_empty() {
            Column::new().push(text("No transactions").color(Color::from_rgb8(77, 77, 77)))
        } else {
            self.transactions
                .iter()
                .fold(Column::new().spacing(15), |col, tx| {
                    let txid_short = format!(
                        "{}...{}",
                        &tx.txid.to_string()[0..8],
                        &tx.txid.to_string()[tx.txid.to_string().len() - 8..]
                    );

                    let mut summary = Row::new().spacing(10).align_y(Alignment::Center);
                    match &tx.error {
                        None => {
                            summary = summary.push(text("✓ ").style(|t| text::success(t)));
                            summary = summary.push(
                                text(format!("Transaction sent {}", txid_short))
                                    .color(Color::from_rgb8(77, 77, 77)),
                            );
                        }
                        Some(_) => {
                            summary = summary.push(text("⚠ ").style(|t| text::danger(t)));
                            summary = summary.push(
                                text(format!("Transaction failed to broadcast {}", txid_short))
                                    .color(Color::from_rgb8(77, 77, 77)),
                            );
                        }
                    }

                    let event_row = if !tx.events.is_empty() {
                        let event_labels: Vec<String> = tx
                            .events
                            .iter()
                            .map(|event| {
                                let kind_str = format!("{:?}", event.kind);
                                let kind = kind_str[0..1].to_uppercase() + &kind_str[1..];
                                match &event.space {
                                    Some(space) => format!("{} {}", kind, space),
                                    None => kind,
                                }
                            })
                            .collect();
                        Some(
                            Row::new()
                                .padding([2, 10])
                                .spacing(10)
                                .align_y(Alignment::Center)
                                .push(
                                    text(format!("Events: {}", event_labels.join(", "))).size(14),
                                ),
                        )
                    } else {
                        None
                    };

                    let error_details = tx.error.as_ref().map(|errors| {
                        Container::new(errors.iter().fold(
                            Column::new().spacing(5).padding([5, 10]),
                            |col, (k, v)| {
                                col.push(
                                    text(format!("{}: {}", k, v))
                                        .color(Color::from_rgb8(77, 77, 77)),
                                )
                            },
                        ))
                        .width(Length::Fill)
                        .style(|theme: &Theme| {
                            let palette = theme.extended_palette();
                            container::Style {
                                background: Some(Color::from_rgb8(255, 201, 201).into()),
                                border: rounded(12),
                                text_color: Some(palette.background.strong.text),
                                ..container::Style::default()
                            }
                        })
                        .padding(10)
                    });

                    let mut tx_col = Column::new()
                        .spacing(8)
                        .push(Container::new(summary).width(Length::Fill));
                    if let Some(event_row) = event_row {
                        tx_col = tx_col.push(
                            Container::new(event_row)
                                .width(Length::Fill)
                                .padding([5, 10])
                                .style(|theme: &Theme| {
                                    let palette = theme.extended_palette();
                                    container::Style {
                                        background: Some(palette.background.weak.color.into()),
                                        border: rounded(8).color(palette.background.weak.color),
                                        text_color: Some(palette.background.strong.text),
                                        ..container::Style::default()
                                    }
                                }),
                        );
                    }
                    if let Some(error_details) = error_details {
                        tx_col = tx_col.push(error_details);
                    }

                    col.push(tx_col)
                })
        };

        content.into()
    }
}
