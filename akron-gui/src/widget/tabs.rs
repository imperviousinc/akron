use iced::{
    Center, Element, Fill, Theme,
    widget::{Row, button, horizontal_space, text},
};

struct Tab<'a, Message> {
    label: &'a str,
    selected: bool,
    on_press: Message,
}

impl<'a, Message: 'a + Clone> From<Tab<'a, Message>> for Element<'a, Message> {
    fn from(tab: Tab<'a, Message>) -> Self {
        button(text(tab.label).size(12).align_x(Center))
            .style(move |theme: &Theme, status: button::Status| {
                let mut style = if tab.selected {
                    button::primary
                } else {
                    button::secondary
                }(theme, status);
                style.border = style.border.rounded(7);
                style
            })
            .on_press(tab.on_press)
            .padding([5, 10])
            .width(Fill)
            .into()
    }
}

pub struct TabsRow<'a, Message>(Vec<Tab<'a, Message>>);

impl<'a, Message: 'a> TabsRow<'a, Message> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_tab(mut self, label: &'a str, selected: bool, on_press: Message) -> Self {
        self.0.push(Tab {
            label,
            selected,
            on_press,
        });
        self
    }
}

impl<'a, Message: 'a + Clone> From<TabsRow<'a, Message>> for Element<'a, Message> {
    fn from(tabs: TabsRow<'a, Message>) -> Self {
        Row::new()
            .push(horizontal_space())
            .extend(tabs.0.into_iter().map(|tab| tab.into()))
            .push(horizontal_space())
            .spacing(10)
            .width(Fill)
            .into()
    }
}
