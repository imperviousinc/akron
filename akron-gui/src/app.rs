use iced::{Color, Element, Subscription, Task, application, theme, window};

use crate::{Config, pages::*};

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
                        text: Color::from_rgb8(77, 77, 77),
                        primary: Color::from_rgb8(247, 147, 26),
                        ..theme::Palette::LIGHT
                    },
                    |pallete| {
                        let mut pallete = theme::palette::Extended::generate(pallete);
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
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            Self::Setup(state) => state.view().map(Message::Setup),
            Self::Main(state) => state.view().map(Message::Main),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if let Self::Main(state) = self {
            state.subscription().map(Message::Main)
        } else {
            Subscription::none()
        }
    }
}
