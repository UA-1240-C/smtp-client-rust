use iced::{Element, Length, Padding};
use iced::widget::{column, row, Button, Container, Text, TextInput, text_editor::{Content, TextEditor, Action}};
use smtp_session::{SmtpMessage, SmtpMessageBuilder};

#[derive(Debug, Clone)]
pub enum HomeMessage {
    Send,
    ChangeUser,
    UpdateRecipient(String),
    UpdateSubject(String),
    UpdateMessage(Action),

    UpdateInfoMessage(String),
}

#[derive(Default)]
pub struct Home {
    page_description : String,
    info_message: String,

    recipient: String,
    subject: String,
    message: Content,
}

impl Home {
    pub fn new() -> Self {
        let mut home = Home::default();
        home.page_description = "SMTP Client".to_string();

        home
    }

    pub fn update(&mut self, message: HomeMessage) {
        match message {
            HomeMessage::UpdateRecipient(recipient) => {
                self.recipient = recipient;
            },
            HomeMessage::UpdateSubject(subject) => {
                self.subject = subject;
            },
            HomeMessage::UpdateMessage(action) => {
                self.message.perform(action);
            },
            HomeMessage::UpdateInfoMessage(info_message) => {
                self.info_message = info_message;
            },
            _ => {}
        }
    }

    pub fn view(&self) -> Element<'_, HomeMessage> {

        Container::new(
            column![
                Text::new(self.page_description.clone()).size(25),

                TextInput::new("Recipient", &self.recipient).on_input(|recipient| HomeMessage::UpdateRecipient(recipient)),
                TextInput::new("Subject", &self.subject).on_input(|subject| HomeMessage::UpdateSubject(subject)),

                column![
                    row![
                        Text::new("Message")
                    ].padding(Padding::from([0, 0, 8, 0])),
                    
                    TextEditor::new(&self.message)
                        .height(Length::from(200))
                        .on_action(|action| HomeMessage::UpdateMessage(action)),
                ].padding(Padding::from([4, 0, 0, 0])),

                row![
                    Button::new("Send").on_press(HomeMessage::Send),
                    Button::new("Change user").on_press(HomeMessage::ChangeUser),

                ].spacing(20.),

                Text::new(self.info_message.clone()).size(12.0).line_height(1.0),
            ].max_width(600)
            .spacing(20.)
            .align_items(iced::alignment::Vertical::Center.into())

        )
        .padding(20.)
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

// external methods

impl Home {
    pub fn get_message_builder(&self) -> SmtpMessageBuilder {
        SmtpMessage::builder()
            .to(&self.recipient.clone())
            .subject(&self.subject.clone())
            .body(&self.message.text())
    }
}