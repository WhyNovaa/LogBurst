use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tokio_postgres::Error;

#[derive(Error, Debug)]
#[error("{error}")]
pub struct ApiError {
    status: StatusCode,
    error: ApiErrorType,
}

impl ApiError {
    pub fn new(status: StatusCode, error: ApiErrorType) -> Self {
        Self { status, error }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, self.error.to_string()).into_response()
    }
}

#[derive(Error, Debug)]
pub enum ApiErrorType {
    #[error("Unknown error")]
    Unknown,
    #[error("Internal server error")]
    Internal,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Missing credentials: {0}")]
    MissingCredentials(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("{0}")]
    Other(String),
}

impl ApiError {
    pub fn bad_request(msg: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error: ApiErrorType::BadRequest(msg.to_string()),
        }
    }
    pub fn not_found(msg: &str) -> Self {
        Self::new(StatusCode::NOT_FOUND, ApiErrorType::Other(msg.to_string()))
    }

    pub fn internal() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, ApiErrorType::Internal)
    }

    pub fn unauthorized(msg: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            error: ApiErrorType::Unauthorized(msg.to_string()),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

pub trait IntoApiError {
    type Ok;
    fn to_api_result(self, status_code: StatusCode, error: ApiErrorType) -> ApiResult<Self::Ok>;
    fn not_found(self, msg: &str) -> ApiResult<Self::Ok>;
    fn internal(self) -> ApiResult<Self::Ok>;
    fn bad_request(self, msg: &str) -> ApiResult<Self::Ok>;
    fn unauthorized(self, msg: &str) -> ApiResult<Self::Ok>;
}

impl<T> IntoApiError for Option<T> {
    type Ok = T;

    fn to_api_result(self, status_code: StatusCode, error: ApiErrorType) -> ApiResult<Self::Ok> {
        match self {
            Some(v) => Ok(v),
            None => Err(ApiError::new(status_code, error)),
        }
    }

    fn not_found(self, msg: &str) -> ApiResult<Self::Ok> {
        match self {
            Some(v) => Ok(v),
            None => Err(ApiError::not_found(msg)),
        }
    }

    fn internal(self) -> ApiResult<Self::Ok> {
        match self {
            Some(v) => Ok(v),
            None => Err(ApiError::internal()),
        }
    }

    fn bad_request(self, msg: &str) -> ApiResult<Self::Ok> {
        match self {
            Some(v) => Ok(v),
            None => Err(ApiError::bad_request(msg)),
        }
    }

    fn unauthorized(self, msg: &str) -> ApiResult<Self::Ok> {
        match self {
            Some(v) => Ok(v),
            None => Err(ApiError::unauthorized(msg)),
        }
    }
}

impl<T, E> IntoApiError for Result<T, E> {
    type Ok = T;

    fn to_api_result(
        self,
        status_code: StatusCode,
        error: ApiErrorType,
    ) -> ApiResult<<Self as IntoApiError>::Ok> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(ApiError::new(status_code, error)),
        }
    }

    fn not_found(self, msg: &str) -> ApiResult<<Self as IntoApiError>::Ok> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(ApiError::bad_request(msg)),
        }
    }

    fn internal(self) -> ApiResult<<Self as IntoApiError>::Ok> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(ApiError::internal()),
        }
    }

    fn bad_request(self, msg: &str) -> ApiResult<<Self as IntoApiError>::Ok> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(ApiError::bad_request(msg)),
        }
    }

    fn unauthorized(self, msg: &str) -> ApiResult<<Self as IntoApiError>::Ok> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(ApiError::unauthorized(msg)),
        }
    }
}

impl From<tokio_postgres::Error> for ApiError {
    fn from(_value: Error) -> Self {
        ApiError::internal()
    }
}