use std::sync::Arc;

use tokio::{io::Result, signal::unix::{signal, SignalKind}, sync::Mutex};
use smtp_session::{SmtpSession, SmtpMessageBuilder};

use std::io::{stdin, stdout, Write};

enum State {
    Start,
    Connected,
    Encrypted,
    Authenticated,
    MessageSent,
    End,
}

#[macro_export]
macro_rules! print_w_flush {
    ($($arg:tt)*) => {
        print!($($arg)*);
        _ = stdout().flush();
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut state = State::Start;
    let session: Arc<Mutex<Option<SmtpSession>>> = Arc::new(Mutex::new(None));
    let cloned_session = session.clone();

    print_w_flush!("Welcome to the SMTP client!\nPress Ctrl+C to exit.\n\r\n");
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::spawn(async move {
        sigint.recv().await;
        if let Some(shadow_session) = cloned_session.lock().await.as_mut() {
            match shadow_session.send_quit_cmd().await {
                Ok(_) => {
                    print_w_flush!("Connection closed!\n");
                }
                Err(err) => {
                    print_w_flush!("Error: {}\n", err);
                }
            }
        }
        std::process::exit(0);
    });

    'main: loop {
        match state {
            State::Start => async {
                print_w_flush!("Enter server: ");
                let mut server = String::new();

                match stdin().read_line(&mut server) {
                    Ok(_) => {
                        server = server.trim().to_string();
                    }
                    Err(err) => {
                        state = State::Start;
                        print_w_flush!("Error: {}\n", err);
                    }
                };

                let shadow_session = SmtpSession::connect(server);

                match shadow_session.await {
                    Ok(shadow_session) => {
                        *session.lock().await = Some(shadow_session);
                        state = State::Connected;
                    }
                    Err(err) => {
                        *session.lock().await = None;
                        state = State::Start;
                        print_w_flush!("Error: {}\n", err);
                    }
                }
            }.await,
            State::Connected => async {
                print_w_flush!("Do you want to encrypt the connection? (y/n): ");

                let mut response = String::new();
                match stdin().read_line(&mut response) {
                    Ok(_) => {
                        response = response.trim().to_string();
                    }
                    Err(err) => {
                        state = State::Connected;
                        print_w_flush!("Error: {}", err);
                    }
                };

                match response.trim() {
                    "y" => {
                        if let Some(shadow_session) = session.lock().await.as_mut() {
                            match shadow_session.encrypt_connection().await {
                                Ok(_) => {
                                    state = State::Encrypted;
                                }
                                Err(err) => {
                                    state = State::Connected;
                                    print_w_flush!("Error: {}", err);
                                }
                            }
                        }
                    }
                    "n" => {
                        state = State::Connected;
                    }
                    _ => {
                        state = State::Connected;
                    }
                }
            }.await,
            State::Encrypted => async {
                print_w_flush!("Enter username: ");

                let mut username = String::new();
                match stdin().read_line(&mut username) {
                    Ok(_) => {
                        username = username.trim().to_string();
                    }
                    Err(err) => {
                        state = State::Encrypted;
                        print_w_flush!("Error: {}", err);
                    }
                };

                print_w_flush!("Enter password: ");
                let mut password = String::new();
                match stdin().read_line(&mut password) {
                    Ok(_) => {
                        password = password.trim().to_string();
                    }
                    Err(err) => {
                        state = State::Encrypted;
                        print_w_flush!("Error: {}", err);
                    }
                };

                print_w_flush!("{username}, {password}");

                if let Some(session) = session.lock().await.as_mut() {
                    match session.authenticate(&username, &password).await {
                        Ok(_) => {
                            state = State::Authenticated;
                        }
                        Err(err) => {
                            state = State::Encrypted;
                            print_w_flush!("Error: {}", err);
                        }
                    }
                }
                            }.await,
            State::Authenticated => async {
                print_w_flush!("Enter sender email: ");

                let mut sender = String::new();
                match stdin().read_line(&mut sender) {
                    Ok(_) => {
                        sender = sender.trim().to_string();
                    }
                    Err(err) => {
                        state = State::Authenticated;
                        print_w_flush!("Error: {}", err);
                    }
                };

                print_w_flush!("Enter recipient email: ");

                let mut recipient = String::new();
                match stdin().read_line(&mut recipient) {
                    Ok(_) => {
                        recipient = recipient.trim().to_string();
                    }
                    Err(err) => {
                        state = State::Authenticated;
                        print_w_flush!("Error: {}", err);
                    }
                };

                print_w_flush!("Enter subject: ");

                let mut subject = String::new();
                match stdin().read_line(&mut subject) {
                    Ok(_) => {
                        subject = subject.trim().to_string();
                    }
                    Err(err) => {
                        state = State::Authenticated;
                        print_w_flush!("Error: {}", err);
                    }
                };

                print_w_flush!("Enter message: ");

                let mut message = String::new();
                match stdin().read_line(&mut message) {
                    Ok(_) => {
                        message = message.trim().to_string();
                    }
                    Err(err) => {
                        state = State::Authenticated;
                        print_w_flush!("Error: {}", err);
                    }
                };

                let message = SmtpMessageBuilder::new()
                    .from(sender.trim())
                    .to(recipient.trim())
                    .subject(subject.trim())
                    .body(message.trim())
                    .build();

                match message {
                    Ok(message) => {
                        if let Some(session) = session.lock().await.as_mut() {
                            match session.send_message(message).await {
                                Ok(_) => {
                                    state = State::MessageSent;
                                }
                                Err(err) => {
                                    state = State::Authenticated;
                                    print_w_flush!("Error: {}", err);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        state = State::Authenticated;
                        print_w_flush!("Error: {}", err);
                    }
                }
            }.await,
            State::MessageSent => async {
                print_w_flush!("Do you want to send another message? (y/n): ");

                let mut response = String::new();
                match stdin().read_line(&mut response) {
                    Ok(_) => {
                        response = response.trim().to_string();
                    }
                    Err(err) => {
                        state = State::MessageSent;
                        print_w_flush!("Error: {}", err);
                    }
                };

                match response.trim() {
                    "y" => {
                        state = State::Authenticated;
                    }
                    "n" => {
                        state = State::End;
                    }
                    _ => {
                        state = State::MessageSent;
                    }
                }
            }.await,
            State::End => {
                print_w_flush!("Goodbye!");
                break 'main;
            },
        }
    };

    if let Some(shadow_session) = session.lock().await.as_mut() {
        match shadow_session.send_quit_cmd().await {
            Ok(_) => {
                print_w_flush!("Connection closed!");
            }
            Err(err) => {
                print_w_flush!("Error: {}", err);
            }
        }
    }

    Ok(())
}