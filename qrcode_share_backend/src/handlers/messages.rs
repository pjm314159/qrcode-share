//! Message HTTP handlers
//!
//! Provides endpoints for sending and retrieving messages in channels.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ApiResponse, AppError};
use crate::models::{CreateMessageRequest, Message, MessageResponse};
use crate::state::AppState;

/// Query parameters for listing messages
#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

/// Response for message list
#[derive(Debug, Serialize)]
pub struct MessageListResponse {
    pub messages: Vec<MessageResponse>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Send a message to a channel
pub async fn send_message(
    State(app_state): State<AppState>,
    Path(channel_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<CreateMessageRequest>,
) -> Result<(StatusCode, Json<ApiResponse<MessageResponse>>), AppError> {
    request.validate().map_err(AppError::ValidationError)?;

    let channel = app_state
        .get_channel(&channel_id)
        .ok_or_else(|| AppError::ChannelNotFound(channel_id.clone()))?;

    if channel.has_password {
        let provided_password = headers
            .get("X-Channel-Password")
            .and_then(|v| v.to_str().ok());

        match (provided_password, channel.password_hash.as_deref()) {
            (None, _) => return Err(AppError::PasswordRequired),
            (Some(pwd), Some(hash)) => {
                let verified = crate::auth::verify_password(pwd, hash).map_err(|e| {
                    AppError::Internal(format!("Password verification failed: {}", e))
                })?;
                if !verified {
                    return Err(AppError::WrongPassword);
                }
            }
            (Some(_), None) => {}
        }
    }

    let link_domain = request.link_domain();

    // Check if link domain is allowed
    if !channel.is_link_allowed(&link_domain) {
        return Err(AppError::LinkDomainNotAllowed {
            domain: link_domain.to_string(),
            allowed: channel.link_limitation.clone().unwrap_or_default(),
        });
    }

    // Generate message ID
    let message_id = Uuid::new_v4().to_string();

    // Create message
    let mut message = Message::new(
        message_id.clone(),
        request.name.clone(),
        request.link.clone(),
        std::time::Duration::from_secs(request.expire_seconds),
    );

    // Set optional fields
    message.message_type = request.message_type.as_ref().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s.as_str().into())
        }
    });
    message.location = request.location.as_ref().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s.as_str().into())
        }
    });

    let message = Arc::new(message);

    // Add message to channel
    channel.add_message(message.clone());

    // Update metrics
    app_state.metrics.inc_messages();

    // Build response
    let response = MessageResponse::from((*message).clone());

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

/// Get all messages in a channel
pub async fn get_messages(
    State(app_state): State<AppState>,
    Path(channel_id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<MessageListResponse>>, AppError> {
    let channel = app_state
        .get_channel(&channel_id)
        .ok_or_else(|| AppError::ChannelNotFound(channel_id.clone()))?;

    if channel.has_password {
        let provided_password = headers
            .get("X-Channel-Password")
            .and_then(|v| v.to_str().ok());

        match (provided_password, channel.password_hash.as_deref()) {
            (None, _) => return Err(AppError::PasswordRequired),
            (Some(pwd), Some(hash)) => {
                let verified = crate::auth::verify_password(pwd, hash).map_err(|e| {
                    AppError::Internal(format!("Password verification failed: {}", e))
                })?;
                if !verified {
                    return Err(AppError::WrongPassword);
                }
            }
            (Some(_), None) => {}
        }
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 100) as usize;

    // Get all non-expired messages
    let all_messages = channel.get_messages();

    // Sort by created_at descending (newest first)
    let mut messages: Vec<Arc<Message>> = all_messages;
    messages.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Apply cursor pagination
    let messages: Vec<Arc<Message>> = if let Some(cursor) = query.cursor {
        // Find the cursor position
        let cursor_pos = messages
            .iter()
            .position(|m| m.id == cursor)
            .map(|p| p + 1)
            .unwrap_or(0);
        messages
            .into_iter()
            .skip(cursor_pos)
            .take(limit + 1)
            .collect()
    } else {
        messages.into_iter().take(limit + 1).collect()
    };

    // Determine if there are more messages
    let has_more = messages.len() > limit;
    let messages: Vec<Arc<Message>> = messages.into_iter().take(limit).collect();

    // Get next cursor
    let next_cursor = messages.last().map(|m| m.id.to_string());

    // Convert to response
    let message_responses: Vec<MessageResponse> = messages
        .into_iter()
        .map(|m| MessageResponse::from((*m).clone()))
        .collect();

    let response = MessageListResponse {
        messages: message_responses,
        next_cursor: if has_more { next_cursor } else { None },
        has_more,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get a single message by ID
pub async fn get_message(
    State(app_state): State<AppState>,
    Path((channel_id, message_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<MessageResponse>>, AppError> {
    let channel = app_state
        .get_channel(&channel_id)
        .ok_or_else(|| AppError::ChannelNotFound(channel_id.clone()))?;

    if channel.has_password {
        let provided_password = headers
            .get("X-Channel-Password")
            .and_then(|v| v.to_str().ok());

        match (provided_password, channel.password_hash.as_deref()) {
            (None, _) => return Err(AppError::PasswordRequired),
            (Some(pwd), Some(hash)) => {
                let verified = crate::auth::verify_password(pwd, hash).map_err(|e| {
                    AppError::Internal(format!("Password verification failed: {}", e))
                })?;
                if !verified {
                    return Err(AppError::WrongPassword);
                }
            }
            (Some(_), None) => {}
        }
    }

    let message = channel
        .get_message(&message_id)
        .ok_or_else(|| AppError::MessageNotFound(message_id.clone()))?;

    // Check if expired
    if message.is_expired() {
        return Err(AppError::MessageNotFound(message_id));
    }

    let response = MessageResponse::from((*message).clone());

    Ok(Json(ApiResponse::success(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_messages_query_defaults() {
        let query = ListMessagesQuery {
            cursor: None,
            limit: None,
        };

        assert!(query.cursor.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_message_list_response_serialization() {
        let response = MessageListResponse {
            messages: vec![],
            next_cursor: None,
            has_more: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"messages\":[]"));
        assert!(json.contains("\"has_more\":false"));
    }

    #[test]
    fn test_create_message_request_validation() {
        let valid = CreateMessageRequest {
            name: "Test".to_string(),
            link: "https://example.com".to_string(),
            message_type: None,
            location: None,
            expire_seconds: 3600,
        };
        assert!(valid.validate().is_ok());

        let empty_name = CreateMessageRequest {
            name: "".to_string(),
            link: "https://example.com".to_string(),
            message_type: None,
            location: None,
            expire_seconds: 3600,
        };
        assert!(empty_name.validate().is_err());

        let invalid_link = CreateMessageRequest {
            name: "Test".to_string(),
            link: "not-a-url".to_string(),
            message_type: None,
            location: None,
            expire_seconds: 3600,
        };
        assert!(invalid_link.validate().is_err());
    }

    #[test]
    fn test_message_link_domain_extraction() {
        let request = CreateMessageRequest {
            name: "Test".to_string(),
            link: "https://example.com/path?query=1".to_string(),
            message_type: None,
            location: None,
            expire_seconds: 3600,
        };

        assert_eq!(request.link_domain(), "example.com");
    }
}
