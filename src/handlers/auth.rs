use axum::extract::State;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::handlers::errors::AuthError;
use crate::models::app::AuthCommandSender;
use crate::models::auth_command::AuthCommand;
use crate::models::http_client::auth_payload::AuthPayload;
use crate::models::http_client::creation_payload::CreationPayload;
use crate::models::http_client::reg_payload::RegPayload;
use crate::models::http_client::role::Role;

pub async fn login(
    State(command_sender): State<AuthCommandSender>,
    Json(payload): Json<AuthPayload>,
) -> Response {
    log::info!("Login endpoint {:?}", payload);

    if payload.login.is_empty() || payload.password.is_empty() {
        return AuthError::MissingCredentials.into_response()
    }

    let command = AuthCommand::Login {
        login: payload.login,
        password: payload.password
    };

    send_command(&command_sender, command).await
}

pub async fn registration(
    State(command_sender): State<AuthCommandSender>,
    Json(payload): Json<RegPayload>,
) -> Response {
    log::info!("Registration endpoint: {:?}", payload);

    if payload.login.is_empty() || payload.password.is_empty() {
        return AuthError::MissingCredentials.into_response()
    }

    let command = AuthCommand::CreateUser {
        login: payload.login,
        password: payload.password,
        // default user's role is 'User'
        role: Role::User,
    };

    send_command(&command_sender, command).await
}

pub async fn create_user(
    State(command_sender): State<AuthCommandSender>,
    Json(payload): Json<CreationPayload>,
) -> Response {
    log::info!("Creation endpoint: {:?}", payload);

    if payload.login.is_empty() || payload.password.is_empty() {
        return AuthError::MissingCredentials.into_response()
    }

   let command = AuthCommand::CreateUser {
        login: payload.login,
        password: payload.password,
        role: Role::from(payload.role_name),
    };

    send_command(&command_sender, command).await
}

/*pub async fn delete_user(
    State(command_sender): State<AuthCommandSender>,
    claims: Claims,
    Path(login): Path<String>,
) -> Response {

}
*/
async fn send_command(command_sender: &AuthCommandSender, command: AuthCommand) -> Response {
    let (one_s, one_r) = tokio::sync::oneshot::channel::<Response>();

    if let Err(e) = command_sender.send((command, one_s)).await {
        log::error!("Failed to send command: {}", e);
        return AuthError::InternalServerError.into_response()
    }

    one_r.await.unwrap_or_else(|e| {
        log::error!("oneshot receive failed: {}", e);
        AuthError::InternalServerError.into_response()
    })
}