use anyhow::{Result, anyhow};
use greetd_ipc::ErrorType;
use iced::Task;
use log::{debug, error, info};
use std::{env, os::unix::net::UnixStream};

use greetd_ipc::{AuthMessageType, Request, Response, codec::SyncCodec};

use super::sessions;
use crate::ui::Message;

#[derive(Debug)]
pub struct GreeterClient {
    session: Option<String>,
    username: String,
    password: String,
}

pub enum GreeterClientMessage {
    Login,
    LoginWithCredentials(String, String),
    LoginResult(Result<LoginResult, String>),
    LoginError(String),
}

impl From<GreeterClientMessage> for Message {
    fn from(value: GreeterClientMessage) -> Self {
        Message::GreeterClient(value)
    }
}

impl GreeterClient {
    pub fn new(session: Option<String>, username: String, password: String) -> Self {
        Self {
            session,
            username,
            password,
        }
    }

    pub fn update(&mut self, msg: GreeterClientMessage) -> Task<GreeterClientMessage> {
        match msg {
            GreeterClientMessage::Login => {
                if self.username.is_empty() && self.password.is_empty() {
                    debug!("username and password not set, not logging in");
                    return Task::none();
                }
                return Task::done(GreeterClientMessage::LoginWithCredentials(
                    self.username.clone(),
                    self.password.clone(),
                ));
            }
            GreeterClientMessage::LoginWithCredentials(username, password) => {
                let session = self.session.clone();
                info!("staring login");
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            login(username, password, session).map_err(|e| e.to_string())
                        })
                        .await
                        .unwrap_or_else(|_| Err("Task panicked".to_string()))
                    },
                    GreeterClientMessage::LoginResult,
                );
            }
            GreeterClientMessage::LoginResult(result) => match result {
                Ok(result) => {
                    if let LoginResult::Failure(msg) = result {
                        return Task::done(GreeterClientMessage::LoginError(msg));
                    }
                }
                Err(e) => {
                    error!("Error when login in: {e}");
                    return Task::done(GreeterClientMessage::LoginError(
                        "Unexpected error".to_string(),
                    ));
                }
            },
            GreeterClientMessage::LoginError(_) => {
                // handled by parent
            }
        }
        Task::none()
    }
}

#[derive(Debug, Clone)]
pub enum LoginResult {
    Success,
    Failure(String),
}

fn login(username: String, password: String, session: Option<String>) -> Result<LoginResult> {
    let mut stream = UnixStream::connect(env::var("GREETD_SOCK")?)?;

    let mut next_request = Request::CreateSession { username };
    let mut starting = false;
    info!("starting loop");
    loop {
        next_request.write_to(&mut stream)?;

        match Response::read_from(&mut stream)? {
            Response::AuthMessage {
                auth_message,
                auth_message_type,
            } => {
                info!("auth message");
                let password = password.clone();
                let response = match auth_message_type {
                    AuthMessageType::Visible => Some(password),
                    AuthMessageType::Secret => Some(password),
                    AuthMessageType::Info => {
                        info!("info: {auth_message}");
                        None
                    }
                    AuthMessageType::Error => {
                        info!("error: {auth_message}");
                        None
                    }
                };

                next_request = Request::PostAuthMessageResponse { response };
            }
            Response::Success => {
                info!("success message");
                if starting {
                    return Ok(LoginResult::Success);
                } else {
                    starting = true;

                    let sessions = match sessions::get_sessions() {
                        Ok(session) => session,
                        Err(e) => return Err(anyhow!("error getting sessions, {e}")),
                    };
                    let session = if let Some(session_name) = &session {
                        sessions.iter().find(|s| &s.name == session_name)
                    } else {
                        sessions.first()
                    };
                    let command = match session {
                        Some(session) => session.get_session_command(),
                        None => return Err(anyhow!("no sessoins available")),
                    };
                    next_request = Request::StartSession {
                        env: command.1,
                        cmd: vec![command.0],
                    }
                }
            }
            Response::Error {
                error_type,
                description,
            } => {
                Request::CancelSession.write_to(&mut stream)?;
                match error_type {
                    ErrorType::AuthError => {
                        return Ok(LoginResult::Failure(
                            "wrong username or password".to_string(),
                        ));
                    }
                    ErrorType::Error => return Err(anyhow!("login error: {description:?}")),
                }
            }
        }
    }
}
