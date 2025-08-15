use crate::widget::text::text_semibold;
use crate::widget::{
    form::text_input,
    icon::{text_icon, Icon},
};
use iced::event::{self, Event};
use iced::keyboard::key;
use iced::widget::{
    button, center, column, container, mouse_area, opaque, row, stack, text, Space, Text,
};
use iced::{border, font, keyboard, widget, Fill, Padding, Shrink, Theme};
use iced::{Color, Element, Subscription, Task};
use serde::Deserialize;

#[derive(Default, Debug)]
pub struct FeeRateSelector {
    show_modal: bool,
    fee_rates: Option<FeeRates>,
    fee_fetch_state: FeeFetchState,
    selected_option: Option<FeeRateOption>,
    selected_fee_rate: Option<u32>,
    custom_fee_rate: String,
}

#[derive(Debug, Clone)]
pub enum FeeRateMessage {
    ShowModal,
    HideModal,
    Event(Event),
    FeeRatesFetched(Result<FeeRates, String>),
    SelectFeeRate(FeeRateOption),
    CustomFeeRate(String),
    ConfirmFeeRate,
    Confirmed(u32),
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeeRates {
    #[serde(rename = "fastestFee")]
    fastest_fee: u32,
    #[serde(rename = "halfHourFee")]
    half_hour_fee: u32,
    #[serde(rename = "hourFee")]
    hour_fee: u32,
}

#[derive(Debug, Default)]
enum FeeFetchState {
    #[default]
    Idle,
    Fetching,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeRateOption {
    Fastest,
    HalfHour,
    Hour,
    Custom,
}

impl FeeRateOption {
    pub const ALL: &'static [Self] = &[Self::Fastest, Self::HalfHour, Self::Hour, Self::Custom];

    fn label(&self) -> &'static str {
        match self {
            Self::Fastest => "Fast",
            Self::HalfHour => "Normal",
            Self::Hour => "Slow",
            Self::Custom => "Custom",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::Fastest => "~10 - 20 minutes",
            Self::HalfHour => "~20 - 60 minutes",
            Self::Hour => "~1 - 2 hours",
            Self::Custom => "Custom",
        }
    }

    fn display_value(
        &self,
        fee_rates: Option<&FeeRates>,
        fee_fetch_state: &FeeFetchState,
    ) -> String {
        match (self, fee_rates, fee_fetch_state) {
            (Self::Custom, _, _) => "Custom".to_string(),
            (_, Some(fee_rates), FeeFetchState::Idle) => {
                format!("{} sat/vB", self.fee_rate(fee_rates))
            }
            _ => "--".to_string(),
        }
    }

    fn fee_rate(&self, fee_rates: &FeeRates) -> u32 {
        match self {
            Self::Fastest => fee_rates.fastest_fee,
            Self::HalfHour => fee_rates.half_hour_fee,
            Self::Hour => fee_rates.hour_fee,
            Self::Custom => 0,
        }
    }
}

impl FeeRateSelector {
    pub fn subscription(&self) -> Subscription<FeeRateMessage> {
        event::listen().map(FeeRateMessage::Event)
    }

    fn fetch_fee_rates() -> Task<FeeRateMessage> {
        Task::perform(
            async {
                match reqwest::get("https://mempool.space/api/v1/fees/recommended").await {
                    Ok(response) => match response.json::<FeeRates>().await {
                        Ok(fee_rates) => Ok(fee_rates),
                        Err(e) => Err(format!("Could not fetch fee rates: {}", e)),
                    },
                    Err(e) => Err(format!("Could not fetch fee rates: {}", e)),
                }
            },
            FeeRateMessage::FeeRatesFetched,
        )
    }

    pub fn update(&mut self, message: FeeRateMessage) -> Task<FeeRateMessage> {
        match message {
            FeeRateMessage::ShowModal => {
                self.show_modal = true;
                self.fee_fetch_state = FeeFetchState::Fetching;
                self.selected_option = Some(FeeRateOption::Fastest);
                Self::fetch_fee_rates()
            }
            FeeRateMessage::HideModal => {
                self.show_modal = false;
                self.custom_fee_rate.clear();
                self.selected_option = Some(FeeRateOption::Fastest);
                self.selected_fee_rate = None;
                Task::none()
            }
            FeeRateMessage::Event(event) => match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Tab),
                    modifiers,
                    ..
                }) => {
                    if modifiers.shift() {
                        widget::focus_previous()
                    } else {
                        widget::focus_next()
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::Escape),
                    ..
                }) => {
                    self.show_modal = false;
                    self.custom_fee_rate.clear();
                    self.selected_option = Some(FeeRateOption::Fastest);
                    self.selected_fee_rate = None;
                    Task::none()
                }
                _ => Task::none(),
            },
            FeeRateMessage::FeeRatesFetched(result) => {
                match result {
                    Ok(fee_rates) => {
                        self.fee_rates = Some(fee_rates.clone());
                        self.fee_fetch_state = FeeFetchState::Idle;
                        self.selected_option = Some(FeeRateOption::Fastest);
                        self.selected_fee_rate = Some(fee_rates.fastest_fee);
                    }
                    Err(e) => {
                        eprintln!("Error fetching fee rates: {}", e);
                        self.fee_fetch_state = FeeFetchState::Failed;
                        self.selected_option = Some(FeeRateOption::Custom);
                        self.selected_fee_rate = self.custom_fee_rate.parse().ok();
                    }
                }
                Task::none()
            }
            FeeRateMessage::SelectFeeRate(option) => {
                self.selected_option = Some(option);
                if option == FeeRateOption::Custom {
                    self.selected_fee_rate = self.custom_fee_rate.parse().ok();
                } else if let (Some(fee_rates), FeeFetchState::Idle) =
                    (&self.fee_rates, &self.fee_fetch_state)
                {
                    self.selected_fee_rate = Some(option.fee_rate(fee_rates));
                } else {
                    self.selected_fee_rate = None;
                }
                Task::none()
            }
            FeeRateMessage::CustomFeeRate(value) => {
                self.custom_fee_rate = value;
                if matches!(self.selected_option, Some(FeeRateOption::Custom)) {
                    self.selected_fee_rate = self.custom_fee_rate.parse().ok();
                }
                Task::none()
            }
            FeeRateMessage::ConfirmFeeRate => {
                if let Some(fee_rate) = self.selected_fee_rate {
                    self.show_modal = false;
                    self.custom_fee_rate.clear();
                    self.selected_option = Some(FeeRateOption::Fastest);
                    self.selected_fee_rate = None;
                    Task::done(FeeRateMessage::Confirmed(fee_rate))
                } else {
                    Task::none()
                }
            }
            FeeRateMessage::Confirmed(_) => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, FeeRateMessage> {
        if self.show_modal {
            let mut fee_content = column![text("Fee rate").size(20)].padding(20).spacing(10);

            let fee_options = FeeRateOption::ALL.iter().fold(column![], |column, option| {
                let is_selected = self.selected_option == Some(*option);
                let display_value =
                    option.display_value(self.fee_rates.as_ref(), &self.fee_fetch_state);

                let icon = if is_selected {
                    text_icon(Icon::CircleDot)
                } else {
                    text_icon(Icon::Circle)
                };
                let label = if *option == FeeRateOption::Custom {
                    row![
                        icon.style(move |theme: &Theme| {
                            let palette = theme.extended_palette();
                            text::Style {
                                color: if is_selected {
                                    Some(palette.primary.strong.color)
                                } else {
                                    Some(palette.background.weak.color)
                                },
                                ..text::Style::default()
                            }
                        })
                        .size(20),
                        column![row![
                            text_semibold(option.label().to_string()).size(16),
                            Space::with_width(Fill),
                        ]
                        .padding(Padding {
                            top: 0.0,
                            right: 0.0,
                            bottom: 0.0,
                            left: 10.0,
                        })]
                    ]
                    .padding(5)
                    .align_y(iced::Center)
                } else {
                    row![
                        icon.style(move |theme: &Theme| {
                            let palette = theme.extended_palette();
                            text::Style {
                                color: if is_selected {
                                    Some(palette.primary.strong.color)
                                } else {
                                    Some(palette.background.weak.color)
                                },
                                ..text::Style::default()
                            }
                        })
                        .size(20),
                        column![
                            row![
                                text_semibold(option.label()).size(16),
                                Space::with_width(Fill),
                                text(display_value).size(18)
                            ],
                            text_light(option.description()).size(14),
                        ]
                        .padding(Padding {
                            top: 0.0,
                            right: 0.0,
                            bottom: 0.0,
                            left: 10.0,
                        })
                    ]
                    .align_y(iced::Center)
                    .padding(5)
                };

                let option_container =
                    container(label)
                        .padding(10)
                        .width(Fill)
                        .style(move |theme: &Theme| {
                            let palette = theme.extended_palette();
                            container::Style {
                                background: Some(if is_selected {
                                    // TODO: may need color adjust
                                    Color::from_rgb8(0xFF, 0xFB, 0xFC).into()
                                } else {
                                    palette.background.base.color.into()
                                }),
                                border: border::rounded(8).width(2).color(if is_selected {
                                    palette.primary.strong.color
                                } else {
                                    palette.background.base.color
                                }),
                                text_color: Some(palette.background.strong.text),
                                ..container::Style::default()
                            }
                        });

                column.push(
                    mouse_area(option_container).on_press(FeeRateMessage::SelectFeeRate(*option)),
                )
            });

            fee_content = fee_content.push(fee_options.spacing(5));

            if matches!(self.selected_option, Some(FeeRateOption::Custom)) {
                fee_content = fee_content.push(
                    column![text_input("Fee rate (sat/vB)", &self.custom_fee_rate)
                        .on_input(FeeRateMessage::CustomFeeRate)
                        .padding(8)]
                    .spacing(5),
                );
            }

            if matches!(self.fee_fetch_state, FeeFetchState::Failed) {
                fee_content =
                    fee_content.push(column![text("Could not load fee rates").size(14)].spacing(5));
            }

            fee_content = fee_content.push(row![
                button(text("Cancel"))
                    .padding(20)
                    .width(Shrink)
                    .style(button::text)
                    // .style(|theme: &Theme, status: button::Status| {
                    //     let mut style = button::secondary(theme, status);
                    //     let p = theme.extended_palette();
                    //     style.border = style.border.rounded(7);
                    //     if matches!(status, button::Status::Active) {
                    //         style.background = Some(p.secondary.base.color.into());
                    //     }
                    //     style
                    // })
                    .on_press(FeeRateMessage::HideModal),
                Space::with_width(Fill),
                button(text("Broadcast transaction"))
                    .padding(20)
                    .width(Shrink)
                    .on_press_maybe(
                        self.selected_fee_rate
                            .filter(|&rate| rate > 0)
                            .map(|_| FeeRateMessage::ConfirmFeeRate)
                    )
                    .style(|theme: &Theme, status: button::Status| {
                        let mut style = button::primary(theme, status);
                        style.border = style.border.rounded(7);
                        style
                    }),
            ]);

            let fee_modal = container(fee_content)
                .width(400)
                .padding(10)
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    container::Style {
                        background: Some(palette.background.weak.color.into()),
                        border: border::rounded(12),
                        ..container::Style::default()
                    }
                });

            stack![opaque(
                mouse_area(center(opaque(fee_modal)).style(|_theme| {
                    container::Style {
                        background: Some(
                            Color {
                                a: 0.8,
                                ..Color::BLACK
                            }
                            .into(),
                        ),
                        ..container::Style::default()
                    }
                }))
                .on_press(FeeRateMessage::HideModal)
            )]
            .into()
        } else {
            column![].into()
        }
    }
}

pub fn text_light<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content).font(font::Font {
        weight: font::Weight::Light,
        ..font::Font::DEFAULT
    })
}
