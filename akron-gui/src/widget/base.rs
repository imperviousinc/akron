use crate::widget::text::error_block;
use iced::widget::{container, scrollable, text, Column};
use iced::{Center, Element, Fill};

pub fn result_column<'a, Message: 'a>(
    error: Option<impl text::IntoFragment<'a>>,
    tx_result: Option<Element<'a, Message>>,
    children: impl IntoIterator<Item = Element<'a, Message>>,
) -> Column<'a, Message> {
    let mut col = Column::new();
    if error.is_some() {
        col = col.push(error_block(error));
    }
    if let Some(tx_result) = tx_result {
        col = col.push(tx_result);
    }
    for child in children {
        col = col.push_maybe(child.into());
    }
    col
}

// centered container with consistent width
pub fn base_container<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    scrollable(
        container(
            container(content.into())
                .padding(40)
                .width(650)
                .align_x(Center),
        )
        .width(Fill)
        .align_x(Center),
    )
    .width(Fill)
    .height(Fill)
    .into()
}
