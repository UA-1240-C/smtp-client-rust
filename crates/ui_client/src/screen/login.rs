
use iced::{alignment, Element, Length};
use iced::widget::{column, row, Button, Container, Space, Text, TextInput};


#[derive(Debug, Clone)]
pub enum LoginMessage {
    ToHome,
    UpdateInfoMessage(String),
    ToggleState,
    UpdateServer(String),
    UpdateLogin(String),
    UpdatePassword(String),
}


#[derive(Debug, Clone, PartialEq, Default)]
pub enum State {
    #[default]
    Login,
    Register
}

#[derive(Default)]
pub struct Login {
    state: State,
    info_message: String,

    server: String,
    login: String,
    password: String,
}

impl Login {
    pub fn new() -> Self {
        Login::default()
    }

    pub fn update(&mut self, message: LoginMessage) {
        match message {
            LoginMessage::UpdateInfoMessage(info_message) => {
                self.info_message = info_message;
            },
            LoginMessage::ToggleState => {
                match self.state {
                    State::Login => {
                        self.state = State::Register;
                    },
                    State::Register => {
                        self.state = State::Login;
                    }
                }
            },
            LoginMessage::UpdateServer(server) => {
                self.server = server;
            },
            LoginMessage::UpdateLogin(login) => {
                self.login = login;
            },
            LoginMessage::UpdatePassword(password) => {
                self.password = password;
            },
            _ => {}

        }
    }

    pub fn view<'a>(&self) -> Element<'a, LoginMessage> {
        // main view container
        Container::new(
            column![

            // text to be displayed on top of the page
            match self.state {
                State::Login => Text::new("Login"),
                State::Register => Text::new("Register"),
            },

            // input fields
            TextInput::new("smtp.gmail.com:587", &self.server).on_input(|server| LoginMessage::UpdateServer(server)),
            TextInput::new("user@gmail.com", &self.login).on_input(|login| LoginMessage::UpdateLogin(login)),
            TextInput::new("password", &self.password).on_input(|password| LoginMessage::UpdatePassword(password)),

            // row with buttons to change the state and to move to the next page
            row![
                Space::with_width(Length::Fill),

                Button::new(
                    match self.state {
                        State::Login => "Login",
                        State::Register => "Register",
                    }
                ).on_press(LoginMessage::ToHome),

                Button::new(
                    match self.state {
                        State::Login => "Don't have an account?",
                        State::Register => "Already have an account?",
                    }
                ).on_press(LoginMessage::ToggleState)
                .style(iced::theme::Button::Secondary),

                Space::with_width(Length::Fill),
            ].align_items(alignment::Horizontal::Center.into()).spacing(20.).width(Length::Fill),

            // status of the login / register attempt
            Text::new(self.info_message.clone())

            ].max_width(400)
            .spacing(20.0)
            .align_items(alignment::Vertical::Center.into()),

        )
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .padding(20.)
        .center_x()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

}

// external methods

impl Login {
    pub fn validate_input(&self) -> Result<(String, String, String), String> {
        if self.server.is_empty() || self.login.is_empty() || self.password.is_empty() {
            return Err("Please fill all the fields".to_string());
        }

        let re = regex::Regex::new(r"^[a-zA-Z0-9.-]+:[0-9]{2,5}$").unwrap();
        if !re.is_match(&self.server) {
            return Err("Invalid server address".to_string());
        }

        let re = regex::Regex::new(r"^[a-zA-Z0-9.%+-]+@[a-zA-Z0-9.-]+.[a-zA-Z]{2,}$").unwrap();
        if !re.is_match(&self.login) {
            return Err("Invalid email address".to_string());
        }

        Ok((self.server.clone().to_string(), self.login.clone().to_string(), self.password.clone().to_string()))

    }

    pub fn get_state(&self) -> State {
        self.state.clone()
    }
}