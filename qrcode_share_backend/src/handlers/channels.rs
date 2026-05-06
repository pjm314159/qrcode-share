//! Channel HTTP handlers
//!
//! Provides CRUD endpoints for channels.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;

use crate::error::{ApiResponse, AppError};
use crate::models::{ChannelResponse, CreateChannelRequest, UpdateChannelRequest};
use crate::state::AppState;

/// Query parameters for listing channels
#[derive(Debug, Deserialize)]
pub struct ListChannelsQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub channel_type: Option<String>,
    pub search: Option<String>,
}

/// Create a new channel
pub async fn create_channel(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ChannelResponse>>), AppError> {
    request.validate().map_err(AppError::ValidationError)?;

    let creator_ip = extract_client_ip(&headers);

    let (channel_id, channel_state) = app_state.create_channel(request, &creator_ip)?;

    let response = ChannelResponse {
        id: channel_id,
        name: channel_state.name.to_string(),
        has_password: channel_state.has_password,
        link_limitation: channel_state.link_limitation.clone(),
        channel_type: channel_state.channel_type.as_ref().map(|s| s.to_string()),
        location: channel_state.location.as_ref().map(|s| s.to_string()),
        teacher: channel_state.teacher.as_ref().map(|s| s.to_string()),
        created_at: channel_state.created_at,
        subscriber_count: channel_state.subscriber_count(),
        message_count: channel_state.message_count(),
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

/// Get a channel by ID
pub async fn get_channel(
    State(app_state): State<AppState>,
    Path(channel_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<ChannelResponse>>, AppError> {
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

    let response = ChannelResponse {
        id: channel_id,
        name: channel.name.to_string(),
        has_password: channel.has_password,
        link_limitation: channel.link_limitation.clone(),
        channel_type: channel.channel_type.as_ref().map(|s| s.to_string()),
        location: channel.location.as_ref().map(|s| s.to_string()),
        teacher: channel.teacher.as_ref().map(|s| s.to_string()),
        created_at: channel.created_at,
        subscriber_count: channel.subscriber_count(),
        message_count: channel.message_count(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// List channels with pagination
pub async fn list_channels(
    State(app_state): State<AppState>,
    Query(query): Query<ListChannelsQuery>,
) -> Json<ApiResponse<ChannelListResponse>> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);

    let channels = app_state.list_channels(page as usize, limit as usize);

    let channel_responses: Vec<ChannelResponse> = channels
        .iter()
        .map(|c| ChannelResponse {
            id: c.id.to_string(),
            name: c.name.to_string(),
            has_password: c.has_password,
            link_limitation: c.link_limitation.clone(),
            channel_type: c.channel_type.as_ref().map(|s| s.to_string()),
            location: c.location.as_ref().map(|s| s.to_string()),
            teacher: c.teacher.as_ref().map(|s| s.to_string()),
            created_at: c.created_at,
            subscriber_count: c.subscriber_count(),
            message_count: c.message_count(),
        })
        .collect();

    let response = ChannelListResponse {
        channels: channel_responses,
        total: app_state.channel_count() as i64,
        page,
        limit,
    };

    Json(ApiResponse::success(response))
}

/// Update a channel
pub async fn update_channel(
    State(app_state): State<AppState>,
    Path(channel_id): Path<String>,
    Json(request): Json<UpdateChannelRequest>,
) -> Result<Json<ApiResponse<ChannelResponse>>, AppError> {
    let channel = app_state
        .get_channel(&channel_id)
        .ok_or_else(|| AppError::ChannelNotFound(channel_id.clone()))?;

    if let Some(ref name) = request.name {
        if name.is_empty() || name.len() > 100 {
            return Err(AppError::ValidationError(
                "Channel name must be 1-100 characters".to_string(),
            ));
        }
    }

    let response = ChannelResponse {
        id: channel_id,
        name: request.name.unwrap_or_else(|| channel.name.to_string()),
        has_password: channel.has_password,
        link_limitation: request.link_limitation.or(channel.link_limitation.clone()),
        channel_type: request
            .channel_type
            .or(channel.channel_type.as_ref().map(|s| s.to_string())),
        location: request
            .location
            .or(channel.location.as_ref().map(|s| s.to_string())),
        teacher: request
            .teacher
            .or(channel.teacher.as_ref().map(|s| s.to_string())),
        created_at: channel.created_at,
        subscriber_count: channel.subscriber_count(),
        message_count: channel.message_count(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Delete a channel
pub async fn delete_channel(
    State(app_state): State<AppState>,
    Path(channel_id): Path<String>,
) -> Result<Json<ApiResponse<DeleteChannelResponse>>, AppError> {
    let deleted = app_state.delete_channel(&channel_id);

    if deleted {
        Ok(Json(ApiResponse::success(DeleteChannelResponse {
            deleted: true,
        })))
    } else {
        Err(AppError::ChannelNotFound(channel_id))
    }
}

fn extract_client_ip(headers: &HeaderMap) -> String {
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(val) = xff.to_str() {
            if let Some(ip) = val.split(',').next() {
                let ip = ip.trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
    }

    if let Some(rip) = headers.get("x-real-ip") {
        if let Ok(val) = rip.to_str() {
            let ip = val.trim();
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }

    "127.0.0.1".to_string()
}

/// Response for delete channel
#[derive(Debug, serde::Serialize)]
pub struct DeleteChannelResponse {
    pub deleted: bool,
}

/// Response for channel list
#[derive(Debug, serde::Serialize)]
pub struct ChannelListResponse {
    pub channels: Vec<ChannelResponse>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::state::AppState;
    use std::sync::Arc;

    fn create_test_app_state() -> AppState {
        let config = Arc::new(Config::from_env().unwrap());
        AppState::new(config)
    }

    fn create_test_request(name: &str) -> CreateChannelRequest {
        CreateChannelRequest {
            name: name.to_string(),
            password: None,
            link_limitation: None,
            channel_type: None,
            location: None,
            teacher: None,
        }
    }

    #[test]
    fn test_list_channels_query_defaults() {
        let query = ListChannelsQuery {
            page: None,
            limit: None,
            channel_type: None,
            search: None,
        };

        assert!(query.page.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_list_channels_query_with_values() {
        let query = ListChannelsQuery {
            page: Some(2),
            limit: Some(50),
            channel_type: Some("lecture".to_string()),
            search: Some("test".to_string()),
        };

        assert_eq!(query.page, Some(2));
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.channel_type, Some("lecture".to_string()));
    }

    #[test]
    fn test_channel_list_response_serialization() {
        let response = ChannelListResponse {
            channels: vec![],
            total: 0,
            page: 1,
            limit: 20,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"channels\":[]"));
        assert!(json.contains("\"total\":0"));
        assert!(json.contains("\"page\":1"));
        assert!(json.contains("\"limit\":20"));
    }

    #[test]
    fn test_delete_channel_response() {
        let response = DeleteChannelResponse { deleted: true };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"deleted\":true"));
    }

    #[test]
    fn test_create_channel_request_validation() {
        let valid = create_test_request("Valid Channel");
        assert!(valid.validate().is_ok());

        let empty_name = CreateChannelRequest {
            name: "".to_string(),
            password: None,
            link_limitation: None,
            channel_type: None,
            location: None,
            teacher: None,
        };
        assert!(empty_name.validate().is_err());

        let long_name = CreateChannelRequest {
            name: "x".repeat(101),
            password: None,
            link_limitation: None,
            channel_type: None,
            location: None,
            teacher: None,
        };
        assert!(long_name.validate().is_err());
    }

    #[test]
    fn test_update_channel_request() {
        let request = UpdateChannelRequest {
            name: Some("New Name".to_string()),
            password: None,
            link_limitation: Some(vec!["example.com".to_string()]),
            channel_type: Some("lecture".to_string()),
            location: Some("Room 101".to_string()),
            teacher: Some("Dr. Smith".to_string()),
        };

        assert_eq!(request.name, Some("New Name".to_string()));
        assert!(request.link_limitation.is_some());
    }

    #[tokio::test]
    async fn test_create_channel_handler() {
        let state = create_test_app_state();
        let request = create_test_request("Test Channel");

        let result = create_channel(axum::extract::State(state), HeaderMap::new(), axum::Json(request)).await;

        assert!(result.is_ok());
        let (status, response) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert!(response.0.success);
        assert!(response.0.data.is_some());
        let data = response.0.data.unwrap();
        assert_eq!(data.name, "Test Channel");
        assert!(!data.has_password);
    }

    #[tokio::test]
    async fn test_get_channel_handler() {
        let state = create_test_app_state();
        let create_req = create_test_request("Test");

        let create_result =
            create_channel(axum::extract::State(state.clone()), HeaderMap::new(), axum::Json(create_req))
                .await
                .unwrap();
        let channel_id = create_result.1 .0.data.unwrap().id;

        let get_result =
            get_channel(axum::extract::State(state), axum::extract::Path(channel_id), HeaderMap::new()).await;

        assert!(get_result.is_ok());
        let response = get_result.unwrap();
        assert!(response.0.success);
        let data = response.0.data.unwrap();
        assert_eq!(data.name, "Test");
    }

    #[tokio::test]
    async fn test_get_channel_not_found() {
        let state = create_test_app_state();

        let result = get_channel(
            axum::extract::State(state),
            axum::extract::Path("nonexistent".to_string()),
            HeaderMap::new(),
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ChannelNotFound(id) => assert_eq!(id, "nonexistent"),
            _ => panic!("Expected ChannelNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_list_channels_handler() {
        let state = create_test_app_state();

        for i in 0..3 {
            let request = create_test_request(&format!("Channel {}", i));
            let _ = create_channel(axum::extract::State(state.clone()), HeaderMap::new(), axum::Json(request)).await;
        }

        let query = ListChannelsQuery {
            page: Some(1),
            limit: Some(10),
            channel_type: None,
            search: None,
        };

        let result = list_channels(axum::extract::State(state), axum::extract::Query(query)).await;

        let response = result.0;
        assert!(response.success);
        let data = response.data.unwrap();
        assert_eq!(data.channels.len(), 3);
        assert_eq!(data.total, 3);
    }

    #[tokio::test]
    async fn test_delete_channel_handler() {
        let state = create_test_app_state();
        let create_req = create_test_request("To Delete");

        let create_result =
            create_channel(axum::extract::State(state.clone()), HeaderMap::new(), axum::Json(create_req))
                .await
                .unwrap();
        let channel_id = create_result.1 .0.data.unwrap().id;

        let delete_result = delete_channel(
            axum::extract::State(state.clone()),
            axum::extract::Path(channel_id.clone()),
        )
        .await;

        assert!(delete_result.is_ok());
        let response = delete_result.unwrap();
        assert!(response.0.data.unwrap().deleted);

        let get_result =
            get_channel(axum::extract::State(state), axum::extract::Path(channel_id), HeaderMap::new()).await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_delete_channel_not_found() {
        let state = create_test_app_state();

        let result = delete_channel(
            axum::extract::State(state),
            axum::extract::Path("nonexistent".to_string()),
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ChannelNotFound(id) => assert_eq!(id, "nonexistent"),
            _ => panic!("Expected ChannelNotFound error"),
        }
    }

    #[test]
    fn test_extract_client_ip_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.1, 70.41.3.18".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), "203.0.113.1");
    }

    #[test]
    fn test_extract_client_ip_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "203.0.113.2".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), "203.0.113.2");
    }

    #[test]
    fn test_extract_client_ip_x_forwarded_for_priority() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.1".parse().unwrap());
        headers.insert("x-real-ip", "203.0.113.2".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), "203.0.113.1");
    }

    #[test]
    fn test_extract_client_ip_default() {
        let headers = HeaderMap::new();
        assert_eq!(extract_client_ip(&headers), "127.0.0.1");
    }

    #[test]
    fn test_extract_client_ip_empty_xff() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), "127.0.0.1");
    }

    #[tokio::test]
    async fn test_update_channel_handler() {
        let state = create_test_app_state();
        let create_req = create_test_request("Original Name");

        let create_result =
            create_channel(axum::extract::State(state.clone()), HeaderMap::new(), axum::Json(create_req))
                .await
                .unwrap();
        let channel_id = create_result.1 .0.data.unwrap().id;

        let update_req = UpdateChannelRequest {
            name: Some("Updated Name".to_string()),
            password: None,
            link_limitation: None,
            channel_type: Some("lecture".to_string()),
            location: None,
            teacher: None,
        };

        let update_result = update_channel(
            axum::extract::State(state),
            axum::extract::Path(channel_id),
            axum::Json(update_req),
        )
        .await;

        assert!(update_result.is_ok());
        let response = update_result.unwrap();
        let data = response.0.data.unwrap();
        assert_eq!(data.name, "Updated Name");
    }
}
