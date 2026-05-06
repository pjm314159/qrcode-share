//! Error types and API response handling
//!
//! This module defines the application error types and their HTTP response mappings.
//! All errors follow a consistent JSON format for client consumption.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error codes for API responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorCode {
    ChannelNotFound,
    ValidationError,
    RateLimitExceeded,
    PasswordRequired,
    WrongPassword,
    ChannelLimitReached,
    MessageTooLarge,
    InvalidLinkFormat,
    LinkDomainNotAllowed,
    MessageNotFound,
    ConnectionLimit,
    ServerOverloaded,
    DatabaseError,
    ChannelMessageLimit,
    UserChannelLimit,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::ChannelNotFound => write!(f, "CHANNEL_NOT_FOUND"),
            ErrorCode::ValidationError => write!(f, "VALIDATION_ERROR"),
            ErrorCode::RateLimitExceeded => write!(f, "RATE_LIMIT_EXCEEDED"),
            ErrorCode::PasswordRequired => write!(f, "PASSWORD_REQUIRED"),
            ErrorCode::WrongPassword => write!(f, "WRONG_PASSWORD"),
            ErrorCode::ChannelLimitReached => write!(f, "CHANNEL_LIMIT_REACHED"),
            ErrorCode::MessageTooLarge => write!(f, "MESSAGE_TOO_LARGE"),
            ErrorCode::InvalidLinkFormat => write!(f, "INVALID_LINK_FORMAT"),
            ErrorCode::LinkDomainNotAllowed => write!(f, "LINK_DOMAIN_NOT_ALLOWED"),
            ErrorCode::MessageNotFound => write!(f, "MESSAGE_NOT_FOUND"),
            ErrorCode::ConnectionLimit => write!(f, "CONNECTION_LIMIT"),
            ErrorCode::ServerOverloaded => write!(f, "SERVER_OVERLOADED"),
            ErrorCode::DatabaseError => write!(f, "DATABASE_ERROR"),
            ErrorCode::ChannelMessageLimit => write!(f, "CHANNEL_MESSAGE_LIMIT"),
            ErrorCode::UserChannelLimit => write!(f, "USER_CHANNEL_LIMIT"),
        }
    }
}

/// Application error types
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded { retry_after_seconds: u64 },

    #[error("Password required")]
    PasswordRequired,

    #[error("Wrong password")]
    WrongPassword,

    #[error("Channel limit reached")]
    ChannelLimitReached,

    #[error("Message too large: max {max} bytes, got {actual} bytes")]
    MessageTooLarge { max: usize, actual: usize },

    #[error("Invalid link format")]
    InvalidLinkFormat,

    #[error("Link domain not allowed: {domain}")]
    LinkDomainNotAllowed {
        domain: String,
        allowed: Vec<String>,
    },

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("Connection limit reached")]
    ConnectionLimit,

    #[error("Server overloaded")]
    ServerOverloaded,

    #[error("Channel message limit reached")]
    ChannelMessageLimit,

    #[error("User channel limit reached")]
    UserChannelLimit,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::ChannelNotFound(_) => StatusCode::NOT_FOUND,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            AppError::PasswordRequired => StatusCode::UNAUTHORIZED,
            AppError::WrongPassword => StatusCode::FORBIDDEN,
            AppError::ChannelLimitReached => StatusCode::TOO_MANY_REQUESTS,
            AppError::MessageTooLarge { .. } => StatusCode::BAD_REQUEST,
            AppError::InvalidLinkFormat => StatusCode::BAD_REQUEST,
            AppError::LinkDomainNotAllowed { .. } => StatusCode::BAD_REQUEST,
            AppError::MessageNotFound(_) => StatusCode::NOT_FOUND,
            AppError::ConnectionLimit => StatusCode::TOO_MANY_REQUESTS,
            AppError::ServerOverloaded => StatusCode::SERVICE_UNAVAILABLE,
            AppError::ChannelMessageLimit => StatusCode::TOO_MANY_REQUESTS,
            AppError::UserChannelLimit => StatusCode::TOO_MANY_REQUESTS,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> ErrorCode {
        match self {
            AppError::ChannelNotFound(_) => ErrorCode::ChannelNotFound,
            AppError::ValidationError(_) => ErrorCode::ValidationError,
            AppError::RateLimitExceeded { .. } => ErrorCode::RateLimitExceeded,
            AppError::PasswordRequired => ErrorCode::PasswordRequired,
            AppError::WrongPassword => ErrorCode::WrongPassword,
            AppError::ChannelLimitReached => ErrorCode::ChannelLimitReached,
            AppError::MessageTooLarge { .. } => ErrorCode::MessageTooLarge,
            AppError::InvalidLinkFormat => ErrorCode::InvalidLinkFormat,
            AppError::LinkDomainNotAllowed { .. } => ErrorCode::LinkDomainNotAllowed,
            AppError::MessageNotFound(_) => ErrorCode::MessageNotFound,
            AppError::ConnectionLimit => ErrorCode::ConnectionLimit,
            AppError::ServerOverloaded => ErrorCode::ServerOverloaded,
            AppError::ChannelMessageLimit => ErrorCode::ChannelMessageLimit,
            AppError::UserChannelLimit => ErrorCode::UserChannelLimit,
            AppError::DatabaseError(_) => ErrorCode::DatabaseError,
            AppError::Internal(_) => ErrorCode::DatabaseError,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::DatabaseError("Row not found".to_string()),
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ErrorResponse {
            code: self.error_code(),
            message: self.to_string(),
            retry_after_seconds: match &self {
                AppError::RateLimitExceeded {
                    retry_after_seconds,
                } => Some(*retry_after_seconds),
                _ => None,
            },
        };

        let body = Json(ApiResponse::<()> {
            success: false,
            data: None,
            error: Some(error_response),
        });

        (status, body).into_response()
    }
}

/// Error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u64>,
}

/// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorResponse>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(code: ErrorCode, message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorResponse {
                code,
                message,
                retry_after_seconds: None,
            }),
        }
    }

    /// Create an error response with retry_after
    pub fn error_with_retry(code: ErrorCode, message: String, retry_after_seconds: u64) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorResponse {
                code,
                message,
                retry_after_seconds: Some(retry_after_seconds),
            }),
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };

        (status, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(
            format!("{}", ErrorCode::ChannelNotFound),
            "CHANNEL_NOT_FOUND"
        );
        assert_eq!(
            format!("{}", ErrorCode::ValidationError),
            "VALIDATION_ERROR"
        );
        assert_eq!(
            format!("{}", ErrorCode::RateLimitExceeded),
            "RATE_LIMIT_EXCEEDED"
        );
    }

    #[test]
    fn test_app_error_status_codes() {
        assert_eq!(
            AppError::ChannelNotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::ValidationError("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::RateLimitExceeded {
                retry_after_seconds: 60
            }
            .status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            AppError::PasswordRequired.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(AppError::WrongPassword.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(
            AppError::ChannelLimitReached.status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            AppError::MessageTooLarge {
                max: 100,
                actual: 200
            }
            .status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::InvalidLinkFormat.status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::MessageNotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::ConnectionLimit.status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            AppError::ServerOverloaded.status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn test_api_response_success() {
        let response: ApiResponse<String> = ApiResponse::success("test data".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("test data".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<()> =
            ApiResponse::error(ErrorCode::ChannelNotFound, "Not found".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        let err = response.error.unwrap();
        assert_eq!(err.code, ErrorCode::ChannelNotFound);
        assert_eq!(err.message, "Not found");
    }

    #[test]
    fn test_api_response_serialization() {
        let response: ApiResponse<String> = ApiResponse::success("test".to_string());
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\":\"test\""));
    }

    #[test]
    fn test_from_sqlx_error() {
        let sqlx_error = sqlx::Error::RowNotFound;
        let app_error: AppError = sqlx_error.into();
        match app_error {
            AppError::DatabaseError(_) => (),
            _ => panic!("Expected DatabaseError variant"),
        }
    }
}
