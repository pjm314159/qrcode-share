//! System HTTP handlers
//!
//! Provides health check and metrics endpoints.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sysinfo::System;

use crate::error::ApiResponse;
use crate::state::AppState;

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: HealthChecks,
}

/// Individual health checks
#[derive(Debug, Serialize)]
pub struct HealthChecks {
    pub memory: MemoryCheck,
    pub channels: ChannelCheck,
}

/// Memory health check
#[derive(Debug, Serialize)]
pub struct MemoryCheck {
    pub used_mb: u64,
    pub limit_mb: u64,
    pub percentage: f64,
    pub healthy: bool,
}

/// Channel health check
#[derive(Debug, Serialize)]
pub struct ChannelCheck {
    pub count: usize,
    pub limit: usize,
    pub percentage: f64,
    pub healthy: bool,
}

/// Metrics response
#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub channels: ChannelMetrics,
    pub messages: MessageMetrics,
    pub connections: ConnectionMetrics,
    pub system: SystemMetrics,
}

/// Channel metrics
#[derive(Debug, Serialize)]
pub struct ChannelMetrics {
    pub total: u64,
    pub active: u64,
}

/// Message metrics
#[derive(Debug, Serialize)]
pub struct MessageMetrics {
    pub total: u64,
    pub per_channel_avg: f64,
}

/// Connection metrics
#[derive(Debug, Serialize)]
pub struct ConnectionMetrics {
    pub active_websocket: usize,
    pub total_subscribers: usize,
}

/// System metrics
#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub memory_used_mb: u64,
    pub memory_limit_mb: u64,
    pub cpu_usage_percent: f64,
    pub uptime_seconds: u64,
}

/// Global start time for uptime calculation
static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

fn get_start_time() -> std::time::Instant {
    *START_TIME.get_or_init(std::time::Instant::now)
}

/// Health check endpoint
pub async fn health_check(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<HealthResponse>>, (StatusCode, Json<ApiResponse<HealthResponse>>)> {
    let mut sys = System::new();
    sys.refresh_memory();

    let used_memory = sys.used_memory() / 1024 / 1024;
    let total_memory = sys.total_memory() / 1024 / 1024;
    let memory_percentage = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    let channel_count = app_state.channel_count();
    let channel_limit = app_state.config.max_channels;
    let channel_percentage = if channel_limit > 0 {
        (channel_count as f64 / channel_limit as f64) * 100.0
    } else {
        0.0
    };

    let memory_healthy = memory_percentage < 90.0;
    let channels_healthy = channel_percentage < 90.0;

    let checks = HealthChecks {
        memory: MemoryCheck {
            used_mb: used_memory,
            limit_mb: total_memory,
            percentage: memory_percentage,
            healthy: memory_healthy,
        },
        channels: ChannelCheck {
            count: channel_count,
            limit: channel_limit,
            percentage: channel_percentage,
            healthy: channels_healthy,
        },
    };

    let status = if memory_healthy && channels_healthy {
        HealthStatus::Healthy
    } else if memory_percentage < 95.0 && channel_percentage < 95.0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };

    let uptime = get_start_time().elapsed().as_secs();

    let response = HealthResponse {
        status: status.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        checks,
    };

    if status == HealthStatus::Unhealthy {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse::success(response)),
        ))
    } else {
        Ok(Json(ApiResponse::success(response)))
    }
}

/// Metrics endpoint
pub async fn metrics_handler(
    State(app_state): State<AppState>,
) -> Json<ApiResponse<MetricsResponse>> {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu();

    let used_memory = sys.used_memory() / 1024 / 1024;
    let total_memory = sys.total_memory() / 1024 / 1024;

    let cpu_usage = sys.global_cpu_info().cpu_usage();

    let metrics = app_state.metrics.clone();

    let channel_count = app_state.channel_count();
    let message_count = metrics.message_count();

    let response = MetricsResponse {
        channels: ChannelMetrics {
            total: metrics.channel_count(),
            active: channel_count as u64,
        },
        messages: MessageMetrics {
            total: message_count,
            per_channel_avg: if channel_count > 0 {
                message_count as f64 / channel_count as f64
            } else {
                0.0
            },
        },
        connections: ConnectionMetrics {
            active_websocket: metrics.active_connections(),
            total_subscribers: metrics.total_subscribers(),
        },
        system: SystemMetrics {
            memory_used_mb: used_memory,
            memory_limit_mb: total_memory,
            cpu_usage_percent: cpu_usage as f64,
            uptime_seconds: get_start_time().elapsed().as_secs(),
        },
    };

    Json(ApiResponse::success(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_serialization() {
        let healthy = HealthStatus::Healthy;
        let json = serde_json::to_string(&healthy).unwrap();
        assert_eq!(json, "\"healthy\"");

        let degraded = HealthStatus::Degraded;
        let json = serde_json::to_string(&degraded).unwrap();
        assert_eq!(json, "\"degraded\"");

        let unhealthy = HealthStatus::Unhealthy;
        let json = serde_json::to_string(&unhealthy).unwrap();
        assert_eq!(json, "\"unhealthy\"");
    }

    #[test]
    fn test_health_response_structure() {
        let response = HealthResponse {
            status: HealthStatus::Healthy,
            version: "0.1.0".to_string(),
            uptime_seconds: 100,
            checks: HealthChecks {
                memory: MemoryCheck {
                    used_mb: 500,
                    limit_mb: 2000,
                    percentage: 25.0,
                    healthy: true,
                },
                channels: ChannelCheck {
                    count: 10,
                    limit: 100,
                    percentage: 10.0,
                    healthy: true,
                },
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"version\":\"0.1.0\""));
        assert!(json.contains("\"uptime_seconds\":100"));
    }

    #[test]
    fn test_metrics_response_structure() {
        let response = MetricsResponse {
            channels: ChannelMetrics {
                total: 10,
                active: 8,
            },
            messages: MessageMetrics {
                total: 100,
                per_channel_avg: 10.0,
            },
            connections: ConnectionMetrics {
                active_websocket: 5,
                total_subscribers: 20,
            },
            system: SystemMetrics {
                memory_used_mb: 500,
                memory_limit_mb: 2000,
                cpu_usage_percent: 25.0,
                uptime_seconds: 3600,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"channels\""));
        assert!(json.contains("\"messages\""));
        assert!(json.contains("\"connections\""));
        assert!(json.contains("\"system\""));
    }

    #[test]
    fn test_memory_check_healthy() {
        let check = MemoryCheck {
            used_mb: 500,
            limit_mb: 2000,
            percentage: 25.0,
            healthy: true,
        };

        assert!(check.healthy);
        assert!(check.percentage < 90.0);
    }

    #[test]
    fn test_channel_check_healthy() {
        let check = ChannelCheck {
            count: 10,
            limit: 100,
            percentage: 10.0,
            healthy: true,
        };

        assert!(check.healthy);
        assert!(check.percentage < 90.0);
    }
}
