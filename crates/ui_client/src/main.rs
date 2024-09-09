use std::sync::Arc;
use login::State;
use tokio::sync::Mutex;
use iced::{Command, Element, executor, Theme};
use iced::widget::container;
use iced::Application;

pub mod screen;
use screen::{login, home};

use smtp_session::{self, SmtpMessage, SmtpSession};
use home::HomeMessage;
use login::LoginMessage;
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
                        let state = page.get_state();
                        let validation_result = page.validate_input();

                        match validation_result {
                            Ok(res) => {
                                let (server, login, password) = res;
                                self.save_user_credentials(login.clone(), password.clone());
                                
                                return App::handle_auth(self.session.clone(), server, login, password, state);
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
                            let builder = page.get_message_builder();
                            let message = builder.from(&self.logged_user.clone().expect("")).build().unwrap();
                            
                            return App::handle_send_message(self.session.clone(), message);
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

// commands
impl App {
    fn handle_auth(session: Arc<Mutex<Option<SmtpSession>>>, server: String, login: String, password: String, state: State) -> Command<Message> {
        Command::perform(tokio::task::spawn(
            async move
            {
                let mut session = session.lock().await;

                if let Ok(mut smtp_session) = SmtpSession::connect(&server).await {
                    smtp_session.encrypt_connection().await?;
                    let result = match state {
                        screen::login::State::Login => smtp_session.authenticate(&login, &password).await,
                        screen::login::State::Register => smtp_session.register(&login, &password).await,
                    };
                    *session = Some(smtp_session);
                    return result;
                    
                }
                else {
                    *session = None;
                    return Err(Error::SmtpResponse("Connection failed".to_string()));
                }
                
            }),
            |result| {
                if let Ok(result) = result {
                    match result {
                        Ok(_) => {
                            Message::GoHome
                        },
                        Err(e) => {
                            match e {
                                Error::SmtpResponse(e) => {
                                    return Message::LoginMsg(LoginMessage::UpdateInfoMessage(format!("Error: \n{}", e).to_string()));
                                },
                                _ => {
                                    return Message::LoginMsg(LoginMessage::UpdateInfoMessage(format!("Error: \r\n{}", e).to_string()));
                                }
                            }
                        }
                    }
                }
                else {
                    return Message::LoginMsg(LoginMessage::UpdateInfoMessage("Connection failed".to_string()));
                }
            }
        )
    }

    fn handle_send_message(session: Arc<Mutex<Option<SmtpSession>>>, message: SmtpMessage) -> Command<Message> {
        Command::perform(tokio::task::spawn(
            async move
            {
                let mut session = session.lock().await;

                if let Some(smtp_session) = session.as_mut() {
                    smtp_session.send_message(message).await?;
                }
                else {
                    return Err(Error::SmtpResponse("Connection failed".to_string()));
                }
                Ok(true)
            }),
            |result| {
                if let Ok(result) = result {
                    match result {
                        Ok(_) => {
                            Message::HomeMsg(HomeMessage::UpdateInfoMessage("Message sent successfully".to_string()))
                        },
                        Err(e) => {
                            Message::HomeMsg(HomeMessage::UpdateInfoMessage(e.to_string()))
                        }
                    }
                }
                else {
                    Message::HomeMsg(HomeMessage::UpdateInfoMessage("Connection failed".to_string()))
                }
            }
        )
    }
}

#[tokio::main]
async fn main() {
    App::run(iced::Settings::default()).unwrap();
}