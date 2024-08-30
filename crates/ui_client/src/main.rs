
use std::sync::Arc;
use tokio::sync::Mutex;
use iced::{Command, Element, executor, Theme};
use iced::widget::container;
use iced::Application;

pub mod screen;
use screen::{login, home};

use smtp_session::{self, SmtpSession};
use error_handler::Error;

pub enum Screen {
    LoginPage(screen::Login),
    HomePage(screen::Home),
}

#[derive(Debug)]
pub enum Message {
    LoginMsg(login::LoginMessage),
    HomeMsg(home::HomeMessage),
    GoHome,
}

struct App {
    screen: Screen,
    session: Arc<Mutex<Option<SmtpSession>>>,
    logged_user: Option<String>,
    logged_user_password: Option<String>,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (App, Command<Self::Message>) {
        (App { screen: Screen::LoginPage(screen::Login::new()), 
            session: Arc::new(Mutex::new(None)), 
            logged_user: None, 
            logged_user_password: None }, 
            Command::none())
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn title(&self) -> String {
        String::from("SMTP Client")
    }


    fn update(&mut self, message: Self::Message) -> iced::Command<Message>{

        match message {

            Message::LoginMsg(msg) => {
                match msg {
                    login::LoginMessage::ToHome => {
                        let Screen::LoginPage(page) = &mut self.screen else { return Command::none(); };
                        let validation_result = page.validate_input();

                        match validation_result {
                            Ok(res) => {
                                let (server, login, password) = res;
                                self.save_user_credentials(login.clone(), password.clone());

                                return App::handle_login(self.session.clone(), server, login, password);

                            },
                            Err(e) => {
                                page.update(login::LoginMessage::UpdateInfoMessage(e));
                                return iced::Command::none();
                                
                            }
                        }
                    },
                    _ => {
                        let Screen::LoginPage(page) = &mut self.screen else { return Command::none(); };
                        page.update(msg);
                    },
                }
            },

            Message::HomeMsg(msg) => {

                match msg {
                    home::HomeMessage::ChangeUser => {
                        self.screen = Screen::LoginPage(screen::Login::new());
                    },
                    home::HomeMessage::Send => {
                        if let Screen::HomePage(page) = &mut self.screen {
                            let (recipient, subject, body) = page.get_message_data();
                            let sender = self.logged_user.clone().unwrap();
                            
                            return App::handle_send_message(self.session.clone(), sender, recipient, subject, body);

                        }
                    },
                    _ => {
                        let Screen::HomePage(page) = &mut self.screen else { return Command::none(); };
                        page.update(msg);
                    },
                }

            },
            Message::GoHome => {
                self.screen = Screen::HomePage(screen::Home::new());
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
       let screen = match &self.screen {
            Screen::LoginPage(pageone) => pageone.view().map(Message::LoginMsg),
            Screen::HomePage(pagetwo) => pagetwo.view().map(Message::HomeMsg),
        };
        container(screen).into()
    }
}

// exteranal functions
impl App {
    fn save_user_credentials(&mut self, login: String, password: String) {
        self.logged_user = Some(login);
        self.logged_user_password = Some(password);
    }
}

// helper functions

impl App {
    fn handle_login(session: Arc<Mutex<Option<SmtpSession>>>, server: String, login: String, password: String) -> Command<Message> {
        Command::perform(tokio::task::spawn(
            async move
            {
                let mut session = session.lock().await;

                if let Ok(mut smtp_session) = SmtpSession::connect(server).await {
                    smtp_session.encrypt_connection().await?;
                    smtp_session.authenticate(&login, &password).await?;
                    *session = Some(smtp_session);
                    
                }
                else {
                    *session = None;
                    return Err::<bool, Error>(Error::Smtp("Connection failed".to_string()));
                }
                Ok::<bool, Error>(true)
                
            }),
            |result| {
                match result.unwrap() {
                    Ok(_) => {
                        println!("<== Connection established successfully ==>");
                        Message::GoHome
                       
                    },
                    Err(e) => {
                        println!("<== Connection failed ==>");
                        Message::HomeMsg(home::HomeMessage::UpdateInfoMessage(e.to_string()))
                    }
                }
            }
        )
    }

    fn handle_send_message(session: Arc<Mutex<Option<SmtpSession>>>, sender: String, recipient: String, subject: String, body: String) -> Command<Message> {
        Command::perform(tokio::task::spawn(
            async move
            {
                let mut session = session.lock().await;

                let message = smtp_session::SmtpMessageBuilder::new()
                    .from(&sender)
                    .to(&recipient)
                    .subject(&subject)
                    .body(&body)
                    .build()
                    .unwrap();

                if let Some(smtp_session) = session.as_mut() {
                    smtp_session.send_message(message).await?;
                }
                else {
                    return Err::<bool, Error>(Error::Smtp("Connection failed".to_string()));
                }
                Ok(true)
            }),
            |result| {
                match result {
                    Ok(_) => {
                        Message::HomeMsg(home::HomeMessage::UpdateInfoMessage("Message sent successfully".to_string()))
                    },
                    Err(e) => {
                        Message::HomeMsg(home::HomeMessage::UpdateInfoMessage(e.to_string()))
                    }
                }
            }
        )
    }
}

#[tokio::main]
async fn main() {
    App::run(iced::Settings::default()).unwrap();
}