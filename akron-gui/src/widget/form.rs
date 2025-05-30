use iced::widget::text;
use iced::{
    widget::{
        button, column, pick_list as _pick_list, text_editor, text_input as _text_input, Button,
        Column, Container, PickList, Text, TextInput,
    },
    Background, Border, Center, Element, Fill, Font, Padding, Theme,
};
use std::borrow::Borrow;
use std::convert::Into;

pub const STANDARD_PADDING_Y: f32 = 20.0;
pub const STANDARD_PADDING_X: f32 = 20.0;
pub const STANDARD_PADDING: Padding = Padding {
    top: STANDARD_PADDING_Y,
    right: STANDARD_PADDING_X,
    bottom: STANDARD_PADDING_Y,
    left: STANDARD_PADDING_X,
};

pub fn text_label(text: &str) -> Text<'_> {
    Text::new(text).size(14)
}

pub fn text_input<'a, Message: Clone + 'a>(
    placeholder: &'a str,
    value: &'a str,
) -> TextInput<'a, Message> {
    _text_input(placeholder, value)
        .font(Font::MONOSPACE)
        .style(|theme: &Theme, status: _text_input::Status| {
            let mut style = _text_input::default(theme, status);
            style.border = style.border.rounded(7);
            style
        })
        .padding(STANDARD_PADDING)
}

pub fn pick_list<
    'a,
    Message: Clone,
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
>(
    options: L,
    selected: Option<V>,
    on_select: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, L, V, Message> {
    _pick_list(options, selected, on_select)
        .style(|theme: &Theme, status: _pick_list::Status| {
            let palette = theme.extended_palette();
            _pick_list::Style {
                background: Background::Color(palette.background.base.color),
                border: Border {
                    radius: 7.0.into(),
                    width: 1.0,
                    color: if status == _pick_list::Status::Hovered {
                        palette.background.base.text
                    } else {
                        palette.background.strong.color
                    },
                },
                .._pick_list::default(theme, status)
            }
        })
        .font(Font::MONOSPACE)
        .width(Fill)
        .padding(STANDARD_PADDING)
}

pub fn submit_button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    on_submit: Option<Message>,
) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    Button::new(content)
        .on_press_maybe(on_submit)
        .padding(STANDARD_PADDING)
        .width(Fill)
        .style(|theme: &Theme, status: button::Status| {
            let mut style = button::primary(theme, status);
            style.border = style.border.rounded(7);
            style
        })
}

pub struct Form<'a, Message> {
    submit_label: &'a str,
    submit_message: Option<Message>,
    elements: Vec<Element<'a, Message>>,
}

impl<'a, Message: Clone + 'a> Form<'a, Message> {
    pub fn new(submit_label: &'a str, submit_message: Option<Message>) -> Self {
        Self {
            submit_label,
            submit_message,
            elements: Vec::new(),
        }
    }

    pub fn add_text_input(
        mut self,
        label: &'a str,
        placeholder: &'a str,
        value: &'a str,
        on_input: impl Fn(String) -> Message + 'a,
    ) -> Self {
        self.elements.push(
            column![
                text_label(label),
                text_input(placeholder, value)
                    .on_input(on_input)
                    .on_submit_maybe(self.submit_message.clone()),
            ]
            .spacing(5)
            .into(),
        );
        self
    }

    pub fn add_text_editor(
        mut self,
        label: &'a str,
        placeholder: &'a str,
        content: &'a text_editor::Content,
        on_action: impl Fn(text_editor::Action) -> Message + 'a,
    ) -> Self {
        self.elements.push(
            column![
                text_label(label),
                text_editor(content)
                    .placeholder(placeholder)
                    .on_action(on_action)
                    .font(Font::MONOSPACE)
                    .padding(10)
                    .height(200)
                    .style(|theme: &Theme, status: text_editor::Status| {
                        let mut style = text_editor::default(theme, status);
                        style.border = style.border.rounded(7);
                        style
                    }),
            ]
            .spacing(5)
            .into(),
        );
        self
    }

    pub fn add_pick_list<
        T: ToString + PartialEq + Clone + 'a,
        L: Borrow<[T]> + 'a,
        V: Borrow<T> + 'a,
    >(
        mut self,
        label: &'a str,
        options: L,
        selected: Option<V>,
        on_select: impl Fn(T) -> Message + 'a,
    ) -> Self {
        self.elements.push(
            column![text_label(label), pick_list(options, selected, on_select)]
                .spacing(5)
                .into(),
        );
        self
    }

    pub fn add_text_button(
        mut self,
        label: &'a str,
        placeholder: &'a str,
        value: &'a str,
        on_press: Message,
    ) -> Self {
        self.elements.push(
            column![
                text_label(label),
                button(Text::new(if value.is_empty() {
                    placeholder
                } else {
                    value
                }))
                .style(move |theme: &Theme, status: button::Status| {
                    let palette = theme.extended_palette();
                    button::Style {
                        border: Border {
                            radius: 7.0.into(),
                            width: 1.0,
                            color: if status == button::Status::Hovered {
                                palette.background.base.text
                            } else {
                                palette.background.strong.color
                            },
                        },
                        text_color: if value.is_empty() {
                            palette.background.strong.color
                        } else {
                            palette.background.base.text
                        },
                        background: Some(palette.background.base.color.into()),
                        ..Default::default()
                    }
                })
                .on_press(on_press)
                .width(Fill)
                .padding(STANDARD_PADDING),
            ]
            .spacing(5)
            .into(),
        );
        self
    }
}

impl<'a, Message: 'a + Clone> From<Form<'a, Message>> for Element<'a, Message> {
    fn from(form: Form<'a, Message>) -> Self {
        Column::from_vec(form.elements)
            .push(
                Container::new(
                    submit_button(
                        text(form.submit_label).width(Fill).align_x(Center),
                        form.submit_message,
                    )
                    .width(Fill),
                )
                .align_x(Center)
                .width(Fill),
            )
            .spacing(10)
            .width(Fill)
            .into()
    }
}
