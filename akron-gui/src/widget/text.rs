use iced::{
    font,
    widget::{container, text, Space, Text},
    Element, Fill, Theme,
};

pub fn text_bold<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content).font(font::Font {
        weight: font::Weight::Bold,
        family: font::Family::Name("Karla"),
        ..font::Font::DEFAULT
    })
}

pub fn text_semibold<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content).font(font::Font {
        family: font::Family::Name("Karla"),
        weight: font::Weight::Semibold,
        ..font::Font::DEFAULT
    })
}

pub fn text_monospace<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content).font(font::Font::MONOSPACE)
}

#[allow(dead_code)]
pub fn text_monospace_bold<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content).font(font::Font {
        weight: font::Weight::Bold,
        ..font::Font::MONOSPACE
    })
}

pub fn text_big<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text_bold(content).size(18)
}

pub fn text_small<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
    text(content).size(14)
}

pub fn error_block<'a, Message: 'a>(
    message: Option<impl text::IntoFragment<'a>>,
) -> Element<'a, Message> {
    match message {
        Some(message) => container(
            text(message)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().danger.base.text),
                })
                .center()
                .width(Fill),
        )
        .style(|theme: &Theme| {
            container::Style::default().background(theme.extended_palette().danger.base.color)
        })
        .width(Fill)
        .padding(10)
        .into(),
        None => Space::new(0, 0).into(),
    }
}
