include!("../../../assets/icons.rs");

use iced::{
    Font, Pixels, Theme,
    widget::{Button, Text, button, text_input},
};

pub fn text_icon<'a>(icon: Icon) -> Text<'a> {
    Text::new(icon.as_char()).font(FONT)
}

pub fn button_icon<'a, Message>(icon: Icon) -> Button<'a, Message> {
    Button::new(text_icon(icon)).style(|theme: &Theme, status: button::Status| {
        let mut style = button::secondary(theme, status);
        style.border = style.border.rounded(7);
        style
    })
}

pub fn text_input_icon(icon: Icon, size: Option<Pixels>, spacing: f32) -> text_input::Icon<Font> {
    text_input::Icon {
        font: FONT,
        code_point: icon.as_char(),
        size,
        spacing,
        side: text_input::Side::Left,
    }
}
