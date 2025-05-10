use crate::{
    client::*,
    widget::{
        form::Form,
        text::{error_block, text_big},
    },
};
use iced::{Element, widget::column};

#[derive(Debug, Default)]
pub struct State {
    slabel: Option<SLabel>,
    event: Option<(String, NostrEvent)>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SLabelSelect(SLabel),
    PathPress,
    SignSubmit,
    EventFileLoaded(Result<Option<(String, NostrEvent)>, String>),
    EventFileSaved(Result<(), String>),
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    FilePick,
    Sign(SLabel, NostrEvent),
}

impl State {
    pub fn update(&mut self, message: Message) -> Action {
        self.error = None;
        match message {
            Message::SLabelSelect(slabel) => {
                self.slabel = Some(slabel);
                Action::None
            }
            Message::PathPress => Action::FilePick,
            Message::SignSubmit => Action::Sign(
                self.slabel.as_ref().unwrap().clone(),
                self.event.as_ref().unwrap().1.clone(),
            ),
            Message::EventFileLoaded(result) => {
                match result {
                    Ok(Some(event_file)) => {
                        self.event = Some(event_file);
                    }
                    Ok(None) => {}
                    Err(err) => self.error = Some(err),
                }
                Action::None
            }
            Message::EventFileSaved(result) => {
                if let Err(err) = result {
                    self.error = Some(err);
                }
                Action::None
            }
        }
    }

    pub fn view<'a>(&'a self, owned_spaces: &'a Vec<SLabel>) -> Element<'a, Message> {
        column![
            text_big("Sign Nostr event"),
            error_block(self.error.as_ref()),
            Form::new(
                "Save",
                (self.slabel.is_some() && self.event.is_some()).then_some(Message::SignSubmit),
            )
            .add_pick_list(
                "Space",
                owned_spaces.as_slice(),
                self.slabel.as_ref(),
                Message::SLabelSelect
            )
            .add_text_button(
                "Nostr event",
                "JSON file",
                self.event.as_ref().map_or("", |p| &p.0),
                Message::PathPress,
            ),
        ]
        .spacing(10)
        .padding([60, 100])
        .into()
    }
}
