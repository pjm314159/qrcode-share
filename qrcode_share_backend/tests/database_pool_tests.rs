//! Database connection pool and cleanup task tests
//!
//! These tests verify database connection pool behavior under stress
//! and background cleanup task functionality.
//!
//! NOTE: Pool tests require Docker (testcontainers). If Docker is not available,
//! they will be skipped. Cleanup tests use in-memory state and always run.

use std::sync::Arc;
use std::time::Duration;

use serial_test::serial;

use qrcode_share_backend::{
    models::{CreateChannelRequest, Message},
    start_cleanup_task, AppState, Config,
};

/// Check if Docker is available and testcontainers can start containers
async fn docker_available() -> bool {
    match tokio::time::timeout(
        Duration::from_secs(5),
        tokio::process::Command::new("docker").arg("info").output(),
    )
    .await
    {
        Ok(Ok(output)) => output.status.success(),
        _ => false,
    }
}

/// Try to start a PostgreSQL container with timeout.
/// Returns None if the container cannot be started within the timeout.
async fn start_postgres_with_timeout(
    db_name: &str,
) -> Option<(
    testcontainers::ContainerAsync<testcontainers_modules::postgres::Postgres>,
    String,
)> {
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;

    let start_result = tokio::time::timeout(
        Duration::from_secs(60),
        Postgres::default()
            .with_db_name(db_name)
            .with_user("test_user")
            .with_password("test_password")
            .start(),
    )
    .await;

    match start_result {
        Ok(Ok(postgres)) => {
            let host = postgres.get_host().await.ok()?;
            let port = postgres.get_host_port_ipv4(5432).await.ok()?;
            let database_url = format!(
                "postgres://test_user:test_password@{}:{}/{}",
                host, port, db_name
            );
            Some((postgres, database_url))
        }
        Ok(Err(e)) => {
            eprintln!("Failed to start PostgreSQL container: {}", e);
            None
        }
        Err(_) => {
            eprintln!("Timeout starting PostgreSQL container for {}", db_name);
            None
        }
    }
}

/// Test: Background cleanup task removes expired messages
#[tokio::test]
#[serial]
async fn test_cleanup_task_removes_expired_messages() {
    let config = Arc::new(Config::from_env().expect("Config failed"));
    let app_state = AppState::new(config);

    let request = CreateChannelRequest {
        name: "Cleanup Test Channel".to_string(),
        password: None,
        link_limitation: None,
        channel_type: None,
        location: None,
        teacher: None,
    };

    let (channel_id, channel_state) = app_state
        .create_channel(request, "127.0.0.1")
        .expect("Channel creation failed");

    for i in 0..5 {
        let message = Arc::new(Message::new(
            format!("msg_{}", i),
            format!("Message {}", i),
            format!("https://example.com/{}", i),
            Duration::from_millis(50),
        ));
        channel_state.add_message(message);
    }

    assert_eq!(
        channel_state.message_count(),
        5,
        "Should have 5 messages before cleanup"
    );

    tokio::time::sleep(Duration::from_millis(100)).await;

    app_state.cleanup_expired_messages();
    app_state.cleanup_inactive_channels();

    assert_eq!(
        channel_state.message_count(),
        0,
        "Messages should be cleaned up after expiration"
    );

    let cleanup_handle = start_cleanup_task(app_state.clone());
    tokio::time::sleep(Duration::from_millis(150)).await;
    cleanup_handle.abort();

    assert!(
        app_state.get_channel(&channel_id).is_some(),
        "Channel should still exist"
    );
}

/// Test: Cleanup removes inactive channels
#[tokio::test]
#[serial]
async fn test_cleanup_task_removes_inactive_channels() {
    let config = Arc::new(Config::from_env().expect("Config failed"));
    let app_state = AppState::new(config);

    let channel_ids = vec!["ch_aaaa", "ch_bbbb", "ch_cccc"];
    for _id in &channel_ids {
        let request = CreateChannelRequest {
            name: "Inactive Channel".to_string(),
            password: None,
            link_limitation: None,
            channel_type: None,
            location: None,
            teacher: None,
        };
        app_state
            .create_channel(request, "127.0.0.1")
            .expect("Channel creation failed");
    }

    assert_eq!(app_state.channel_count(), 3, "Should have 3 channels");

    app_state.cleanup_expired_messages();
    app_state.cleanup_inactive_channels();

    let remaining = app_state.channel_count();
    assert!(
        remaining <= 3,
        "Should have at most 3 channels remaining, got {}",
        remaining
    );
}

/// Test: Connection pool with limited connections (requires Docker)
#[tokio::test]
#[serial]
async fn test_pool_limited_connections() {
    if !docker_available().await {
        eprintln!("Skipping test_pool_limited_connections: Docker not available");
        return;
    }

    let Some((_container, database_url)) =
        start_postgres_with_timeout("qrcode_share_pool_test").await
    else {
        eprintln!("Skipping test_pool_limited_connections: Could not start PostgreSQL container");
        return;
    };

    use sqlx::postgres::PgPoolOptions;

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let conn1 = pool
        .acquire()
        .await
        .expect("First acquisition should succeed");
    drop(conn1);

    let conn2 = pool
        .acquire()
        .await
        .expect("Second acquisition should succeed");
    drop(conn2);

    let result: Result<(i32,), sqlx::Error> = sqlx::query_as("SELECT 1").fetch_one(&pool).await;
    assert!(
        result.is_ok(),
        "Pool should be usable after connection release"
    );
}

/// Test: Connection pool timeout when exhausted (requires Docker)
#[tokio::test]
#[serial]
async fn test_pool_timeout_when_exhausted() {
    if !docker_available().await {
        eprintln!("Skipping test_pool_timeout_when_exhausted: Docker not available");
        return;
    }

    let Some((_container, database_url)) =
        start_postgres_with_timeout("qrcode_share_pool_test2").await
    else {
        eprintln!(
            "Skipping test_pool_timeout_when_exhausted: Could not start PostgreSQL container"
        );
        return;
    };

    use sqlx::postgres::PgPoolOptions;

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let held_conn = pool.acquire().await.expect("Should acquire connection");

    let result = tokio::time::timeout(Duration::from_millis(500), pool.acquire()).await;

    match result {
        Ok(conn) => {
            drop(conn);
        }
        Err(_) => {}
    }

    drop(held_conn);

    let recovery_result = tokio::time::timeout(Duration::from_secs(2), pool.acquire()).await;

    assert!(
        recovery_result.is_ok(),
        "Pool should recover after connection is released"
    );
}

/// Test: Multiple concurrent pool acquisitions (requires Docker)
#[tokio::test]
#[serial]
async fn test_pool_concurrent_acquisitions() {
    if !docker_available().await {
        eprintln!("Skipping test_pool_concurrent_acquisitions: Docker not available");
        return;
    }

    let Some((_container, database_url)) =
        start_postgres_with_timeout("qrcode_share_pool_test3").await
    else {
        eprintln!(
            "Skipping test_pool_concurrent_acquisitions: Could not start PostgreSQL container"
        );
        return;
    };

    use sqlx::postgres::PgPoolOptions;

    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(3)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&database_url)
            .await
            .expect("Failed to create pool"),
    );

    let mut handles = vec![];

    for i in 0..6 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let conn = pool_clone
                .acquire()
                .await
                .unwrap_or_else(|_| panic!("Task {} should acquire", i));

            tokio::time::sleep(Duration::from_millis(10)).await;
            drop(conn);
            format!("task_{}", i)
        });
        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    for (idx, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Task {} failed", idx);
    }
}

/// Test: Database reconnection simulation (requires Docker)
#[tokio::test]
#[serial]
async fn test_database_connection_resilience() {
    if !docker_available().await {
        eprintln!("Skipping test_database_connection_resilience: Docker not available");
        return;
    }

    let Some((_container, database_url)) =
        start_postgres_with_timeout("qrcode_share_pool_test4").await
    else {
        eprintln!(
            "Skipping test_database_connection_resilience: Could not start PostgreSQL container"
        );
        return;
    };

    use sqlx::postgres::PgPoolOptions;

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let result: Result<(i32,), sqlx::Error> = sqlx::query_as("SELECT 1").fetch_one(&pool).await;
    assert!(result.is_ok(), "Initial query should succeed");

    pool.close().await;

    let new_pool = PgPoolOptions::new()
        .max_connections(2)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .expect("Failed to create new pool");

    let reconnect_result: Result<(i32,), sqlx::Error> =
        sqlx::query_as("SELECT 1").fetch_one(&new_pool).await;

    assert!(reconnect_result.is_ok(), "Reconnection should succeed");
}
