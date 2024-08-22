use iced::{Command, Element, executor, Theme};
use iced::widget::container;
use iced::Application;

pub mod screen;
use screen::{login, home };


pub enum Screen {
    LoginPage(screen::Login),
    HomePage(screen::Home),
}

#[derive(Debug)]
pub enum Message {
    LoginMsg(login::LoginMessage),
    HomeMsg(home::HomeMessage),
}

struct App {
    screen: Screen,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (App, Command<Self::Message>) {
        (App { screen: Screen::LoginPage(screen::Login::new())} , Command::none())
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
                        let result = page.validate_input();

                        match result {
                            Ok(_) => {
                                self.screen = Screen::HomePage(screen::Home::new());
                            },
                            Err(e) => {
                                page.update(login::LoginMessage::UpdateInfoMessage(e));
                            }
                        }

                        // make async login attempt here

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
                            
                            // make async send attempt here

                            page.update(home::HomeMessage::UpdateInfoMessage("Sending message...".to_string()));

                        }
                    },
                    _ => {
                        let Screen::HomePage(page) = &mut self.screen else { return Command::none(); };
                        page.update(msg);
                    },
                }
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

fn main() -> iced::Result {
    App::run(iced::Settings::default())
}
