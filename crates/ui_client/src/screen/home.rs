use iced::{Element, Length};
use iced::widget::{column, row, Button, Container, Text, TextInput};

#[derive(Debug, Clone)]
pub enum HomeMessage {
    Send,
    ChangeUser,
    UpdateRecipient(String),
    UpdateSubject(String),
    UpdateMessage(String),

    UpdateInfoMessage(String),
}

#[derive(Default)]
pub struct Home {
    page_description : String,
    info_message: String,

    recipient: String,
    subject: String,
    message: String,
}

impl Home {
    pub fn new() -> Self {
        Home::default()
    }

    pub fn update(&mut self, message: HomeMessage) {
        match message {
            HomeMessage::UpdateRecipient(recipient) => {
                self.recipient = recipient;
            },
            HomeMessage::UpdateSubject(subject) => {
                self.subject = subject;
            },
            HomeMessage::UpdateMessage(message) => {
                self.message = message;
            },
            HomeMessage::UpdateInfoMessage(info_message) => {
                self.info_message = info_message;
            },
            _ => {}
        }
    }

    pub fn view<'a>(&self) -> Element<'a, HomeMessage> {
        Container::new(
            column![
                Text::new(self.page_description.clone()),

                TextInput::new("Recipient", &self.recipient).on_input(|recipient| HomeMessage::UpdateRecipient(recipient)),
                TextInput::new("Subject", &self.subject).on_input(|subject| HomeMessage::UpdateSubject(subject)),
                TextInput::new("Message", &self.message).on_input(|message| HomeMessage::UpdateMessage(message)),

                row![
                    Button::new("Send").on_press(HomeMessage::Send),
                    Button::new("Change user").on_press(HomeMessage::ChangeUser),

                ].spacing(20.),

                Text::new(self.info_message.clone())
            ].max_width(400.)
            .spacing(20.)
        )
        .padding(20.)
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
