use crate::{pages::*, Config};
use iced::{application, theme, window, Color, Element, Font, Subscription, Task};

#[derive(Debug)]
pub enum State {
    Setup(setup::State),
    Main(main::State),
}

#[derive(Debug)]
enum Message {
    Setup(setup::Message),
    Main(main::Message),
}

impl State {
    pub fn run(config: Config) -> iced::Result {
        let (state, task) = setup::State::run(config);
        let state = Self::Setup(state);
        let task = task.map(Message::Setup);
        application("Akron", Self::update, Self::view)
            .font(include_bytes!("../../assets/icons.ttf").as_slice())
            .font(include_bytes!("../../assets/fonts/Karla/static/Karla-Bold.ttf").as_slice())
            .font(include_bytes!("../../assets/fonts/Karla/static/Karla-SemiBold.ttf").as_slice())
            .font(include_bytes!("../../assets/fonts/Karla/static/Karla-Regular.ttf").as_slice())
            .default_font(Font {
                family: iced::font::Family::Name("Karla"),
                weight: iced::font::Weight::Normal,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            })
            .subscription(Self::subscription)
            .window(window::Settings {
                min_size: Some((1300.0, 500.0).into()),
                icon: Some(
                    window::icon::from_rgba(
                        include_bytes!("../../assets/akron.rgba").to_vec(),
                        64,
                        64,
                    )
                    .expect("Failed to load icon"),
                ),
                ..Default::default()
            })
            .theme(|_| {
                theme::Theme::custom_with_fn(
                    "Bitcoin".into(),
                    theme::Palette {
                        text: Color::from_rgb8(0, 0, 0),
                        primary: Color::from_rgb8(0xFD, 0x9E, 0xB2),
                        ..theme::Palette::LIGHT
                    },
                    |pallete| {
                        let mut pallete = theme::palette::Extended::generate(pallete);
                        pallete.primary.base.text = Color::from_rgb8(0xFD, 0x9E, 0xB2);
                        pallete.primary.base.text = Color::WHITE;
                        pallete.primary.strong.text = Color::WHITE;
                        pallete.primary.weak.text = Color::WHITE;
                        pallete
                    },
                )
            })
            .run_with(move || (state, task))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match (&mut *self, message) {
            (Self::Setup(state), Message::Setup(message)) => match state.update(message) {
                setup::Action::Return(config, client) => {
                    let (state, task) = main::State::run(config, client);
                    let task = task.map(Message::Main);
                    *self = Self::Main(state);
                    task
                }
                setup::Action::Task(task) => task.map(Message::Setup),
            },
            (Self::Main(state), Message::Main(message)) => match state.update(message) {
                main::Action::Return(mut config) => {
                    config.reset();
                    let (state, task) = setup::State::run(config);
                    let task = task.map(Message::Setup);
                    *self = Self::Setup(state);
                    task
                }
                main::Action::Task(task) => task.map(Message::Main),
            },
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match self {
            Self::Setup(state) => state.view().map(Message::Setup),
            Self::Main(state) => state.view().map(Message::Main),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match self {
            Self::Setup(state) => state.subscription().map(Message::Setup),
            Self::Main(state) => state.subscription().map(Message::Main),
        }
    }
}
