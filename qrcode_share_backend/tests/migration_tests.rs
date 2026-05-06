//! Database migration tests for QRcode Share Backend
//!
//! These tests use testcontainers to spin up a PostgreSQL container,
//! run migrations, and verify the database schema.

use std::time::Duration;

use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

use qrcode_share_backend::Database;

/// Check if Docker is available for testcontainers
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

/// Start a PostgreSQL container and return the database URL.
/// Returns None if the container cannot be started within timeout.
async fn start_postgres() -> Option<(String, testcontainers::ContainerAsync<Postgres>)> {
    let start_result = tokio::time::timeout(
        Duration::from_secs(60),
        Postgres::default()
            .with_db_name("qrcode_share_test")
            .with_user("test_user")
            .with_password("test_password")
            .start(),
    )
    .await;

    let postgres = match start_result {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => {
            eprintln!("Failed to start PostgreSQL container: {}", e);
            return None;
        }
        Err(_) => {
            eprintln!("Timeout starting PostgreSQL container");
            return None;
        }
    };

    let host = postgres.get_host().await.ok()?;
    let port = postgres.get_host_port_ipv4(5432).await.ok()?;

    let database_url = format!(
        "postgres://test_user:test_password@{}:{}/qrcode_share_test",
        host, port
    );

    Some((database_url, postgres))
}

/// Create a database connection pool from URL (used internally by some tests)
#[allow(dead_code)]
async fn create_pool(database_url: &str) -> sqlx::PgPool {
    PgPoolOptions::new()
        .max_connections(3)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .expect("Failed to create pool")
}

/// Test: Run all migrations on a fresh database
#[tokio::test]
#[serial]
async fn test_migration_fresh_database() {
    if !docker_available().await {
        eprintln!("Skipping test_migration_fresh_database: Docker not available");
        return;
    }

    let container_result = start_postgres().await;
    if container_result.is_none() {
        eprintln!("Skipping test_migration_fresh_database: Could not start PostgreSQL container");
        return;
    }
    let (database_url, _container) = container_result.unwrap();

    let database = Database::new_from_url(&database_url).await;

    database
        .run_migrations()
        .await
        .expect("Migrations should succeed on fresh database");

    let pool = database.pool();
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to query tables");

    let table_names: Vec<&str> = tables.iter().map(|(n,)| n.as_str()).collect();
    assert!(
        table_names.contains(&"channels"),
        "Expected 'channels' table, found: {:?}",
        table_names
    );
}

/// Test: Verify channels table schema after migration
#[tokio::test]
#[serial]
async fn test_migration_channels_schema() {
    if !docker_available().await {
        eprintln!("Skipping test_migration_channels_schema: Docker not available");
        return;
    }

    let container_result = start_postgres().await;
    if container_result.is_none() {
        eprintln!("Skipping test_migration_channels_schema: Could not start PostgreSQL container");
        return;
    }
    let (database_url, _container) = container_result.unwrap();

    let database = Database::new_from_url(&database_url).await;
    database.run_migrations().await.expect("Migrations failed");

    let pool = database.pool();

    let columns: Vec<(String, String)> = sqlx::query_as(
        r#"SELECT column_name, data_type FROM information_schema.columns
           WHERE table_name = 'channels' AND table_schema = 'public'
           ORDER BY ordinal_position"#,
    )
    .fetch_all(pool)
    .await
    .expect("Failed to query columns");

    let column_map: std::collections::HashMap<&str, &str> = columns
        .iter()
        .map(|(name, dtype)| (name.as_str(), dtype.as_str()))
        .collect();

    assert!(column_map.contains_key("id"), "Missing 'id' column");
    assert!(column_map.contains_key("name"), "Missing 'name' column");
    assert!(
        column_map.contains_key("password_hash"),
        "Missing 'password_hash' column"
    );
    assert!(
        column_map.contains_key("link_limitation"),
        "Missing 'link_limitation' column"
    );
    assert!(
        column_map.contains_key("creator_ip"),
        "Missing 'creator_ip' column"
    );
    assert!(
        column_map.contains_key("message_count"),
        "Missing 'message_count' column"
    );
    assert!(
        column_map.contains_key("last_activity"),
        "Missing 'last_activity' column"
    );
    assert!(
        column_map.contains_key("created_at"),
        "Missing 'created_at' column"
    );

    assert_eq!(
        column_map.get("id").copied(),
        Some("character varying"),
        "id should be varchar"
    );
    assert_eq!(
        column_map.get("name").copied(),
        Some("character varying"),
        "name should be varchar"
    );
    assert_eq!(
        column_map.get("link_limitation").copied(),
        Some("jsonb"),
        "link_limitation should be jsonb"
    );
    assert_eq!(
        column_map.get("message_count").copied(),
        Some("integer"),
        "message_count should be integer"
    );
}

/// Test: Verify indexes are created after migration
#[tokio::test]
#[serial]
async fn test_migration_indexes() {
    if !docker_available().await {
        eprintln!("Skipping test_migration_indexes: Docker not available");
        return;
    }

    let container_result = start_postgres().await;
    if container_result.is_none() {
        eprintln!("Skipping test_migration_indexes: Could not start PostgreSQL container");
        return;
    }
    let (database_url, _container) = container_result.unwrap();

    let database = Database::new_from_url(&database_url).await;
    database.run_migrations().await.expect("Migrations failed");

    let pool = database.pool();

    let indexes: Vec<(String,)> = sqlx::query_as(
        r#"SELECT indexname FROM pg_indexes
           WHERE tablename = 'channels' AND schemaname = 'public'
           ORDER BY indexname"#,
    )
    .fetch_all(pool)
    .await
    .expect("Failed to query indexes");

    let index_names: Vec<&str> = indexes.iter().map(|(n,)| n.as_str()).collect();

    assert!(
        index_names.iter().any(|i| i.contains("channels_pkey")),
        "Expected primary key index, found: {:?}",
        index_names
    );

    assert!(
        index_names.iter().any(|i| i.contains("last_activity")),
        "Expected last_activity index, found: {:?}",
        index_names
    );

    assert!(
        index_names.iter().any(|i| i.contains("creator_ip")),
        "Expected creator_ip index, found: {:?}",
        index_names
    );
}

/// Test: Verify cleanup function exists after migration
#[tokio::test]
#[serial]
async fn test_migration_cleanup_function() {
    if !docker_available().await {
        eprintln!("Skipping test_migration_cleanup_function: Docker not available");
        return;
    }

    let container_result = start_postgres().await;
    if container_result.is_none() {
        eprintln!("Skipping test_migration_cleanup_function: Could not start PostgreSQL container");
        return;
    }
    let (database_url, _container) = container_result.unwrap();

    let database = Database::new_from_url(&database_url).await;
    database.run_migrations().await.expect("Migrations failed");

    let pool = database.pool();

    let functions: Vec<(String,)> = sqlx::query_as(
        r#"SELECT routine_name FROM information_schema.routines
           WHERE routine_schema = 'public' AND routine_type = 'FUNCTION'
           ORDER BY routine_name"#,
    )
    .fetch_all(pool)
    .await
    .expect("Failed to query functions");

    let function_names: Vec<&str> = functions.iter().map(|(n,)| n.as_str()).collect();

    assert!(
        function_names.contains(&"cleanup_inactive_channels"),
        "Expected cleanup_inactive_channels function, found: {:?}",
        function_names
    );

    let result: (i32,) = sqlx::query_as("SELECT cleanup_inactive_channels()")
        .fetch_one(pool)
        .await
        .expect("Failed to call cleanup function");

    assert_eq!(
        result.0, 0,
        "Cleanup should delete 0 channels on empty database"
    );
}

/// Test: Verify materialized view exists after migration
#[tokio::test]
#[serial]
async fn test_migration_stats_view() {
    if !docker_available().await {
        eprintln!("Skipping test_migration_stats_view: Docker not available");
        return;
    }

    let container_result = start_postgres().await;
    if container_result.is_none() {
        eprintln!("Skipping test_migration_stats_view: Could not start PostgreSQL container");
        return;
    }
    let (database_url, _container) = container_result.unwrap();

    let database = Database::new_from_url(&database_url).await;
    database.run_migrations().await.expect("Migrations failed");

    let pool = database.pool();

    let views: Vec<(String,)> =
        sqlx::query_as(r#"SELECT matviewname FROM pg_matviews WHERE schemaname = 'public'"#)
            .fetch_all(pool)
            .await
            .expect("Failed to query materialized views");

    let view_names: Vec<&str> = views.iter().map(|(n,)| n.as_str()).collect();

    assert!(
        view_names.contains(&"channel_stats"),
        "Expected channel_stats materialized view, found: {:?}",
        view_names
    );

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM channel_stats")
        .fetch_one(pool)
        .await
        .expect("Failed to query channel_stats view");

    assert_eq!(
        count.0, 0,
        "channel_stats should be empty on fresh database"
    );

    sqlx::query("REFRESH MATERIALIZED VIEW channel_stats")
        .execute(pool)
        .await
        .expect("Failed to refresh materialized view");
}

/// Test: Migrations are idempotent (running twice should not fail)
#[tokio::test]
#[serial]
async fn test_migration_idempotent() {
    if !docker_available().await {
        eprintln!("Skipping test_migration_idempotent: Docker not available");
        return;
    }

    let container_result = start_postgres().await;
    if container_result.is_none() {
        eprintln!("Skipping test_migration_idempotent: Could not start PostgreSQL container");
        return;
    }
    let (database_url, _container) = container_result.unwrap();

    let database = Database::new_from_url(&database_url).await;

    database
        .run_migrations()
        .await
        .expect("First migration run should succeed");

    database
        .run_migrations()
        .await
        .expect("Second migration run should succeed (idempotent)");
}

/// Test: Insert and query a channel after migration
#[tokio::test]
#[serial]
async fn test_migration_channel_crud() {
    if !docker_available().await {
        eprintln!("Skipping test_migration_channel_crud: Docker not available");
        return;
    }

    let container_result = start_postgres().await;
    if container_result.is_none() {
        eprintln!("Skipping test_migration_channel_crud: Could not start PostgreSQL container");
        return;
    }
    let (database_url, _container) = container_result.unwrap();

    let database = Database::new_from_url(&database_url).await;
    database.run_migrations().await.expect("Migrations failed");

    let pool = database.pool();

    sqlx::query(
        r#"INSERT INTO channels (id, name, creator_ip, link_limitation)
           VALUES ($1, $2, $3, $4)"#,
    )
    .bind("test1234")
    .bind("Test Channel")
    .bind("127.0.0.1")
    .bind(serde_json::json!(["example.com"]))
    .execute(pool)
    .await
    .expect("Failed to insert channel");

    let row: (String, String, serde_json::Value) =
        sqlx::query_as("SELECT id, name, link_limitation FROM channels WHERE id = $1")
            .bind("test1234")
            .fetch_one(pool)
            .await
            .expect("Failed to query channel");

    assert_eq!(row.0, "test1234");
    assert_eq!(row.1, "Test Channel");

    let limitation: Vec<String> =
        serde_json::from_value(row.2).expect("Failed to parse link_limitation");
    assert_eq!(limitation, vec!["example.com"]);

    let deleted: (i32,) = sqlx::query_as("SELECT cleanup_inactive_channels()")
        .fetch_one(pool)
        .await
        .expect("Failed to call cleanup");
    assert_eq!(deleted.0, 0, "Active channel should not be cleaned up");

    sqlx::query("REFRESH MATERIALIZED VIEW channel_stats")
        .execute(pool)
        .await
        .expect("Failed to refresh view");

    let stats_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM channel_stats")
        .fetch_one(pool)
        .await
        .expect("Failed to query stats");
    assert_eq!(stats_count.0, 1, "Stats should show 1 channel");
}

/// Test: Verify pg_trgm extension is installed
#[tokio::test]
#[serial]
async fn test_migration_pg_trgm_extension() {
    if !docker_available().await {
        eprintln!("Skipping test_migration_pg_trgm_extension: Docker not available");
        return;
    }

    let container_result = start_postgres().await;
    if container_result.is_none() {
        eprintln!(
            "Skipping test_migration_pg_trgm_extension: Could not start PostgreSQL container"
        );
        return;
    }
    let (database_url, _container) = container_result.unwrap();

    let database = Database::new_from_url(&database_url).await;
    database.run_migrations().await.expect("Migrations failed");

    let pool = database.pool();

    let extensions: Vec<(String,)> =
        sqlx::query_as("SELECT extname FROM pg_extension WHERE extname = 'pg_trgm'")
            .fetch_all(pool)
            .await
            .expect("Failed to query extensions");

    assert!(
        !extensions.is_empty(),
        "pg_trgm extension should be installed"
    );
}

/// Test: Verify NOT NULL constraints on channels table
#[tokio::test]
#[serial]
async fn test_migration_not_null_constraints() {
    if !docker_available().await {
        eprintln!("Skipping test_migration_not_null_constraints: Docker not available");
        return;
    }

    let container_result = start_postgres().await;
    if container_result.is_none() {
        eprintln!(
            "Skipping test_migration_not_null_constraints: Could not start PostgreSQL container"
        );
        return;
    }
    let (database_url, _container) = container_result.unwrap();

    let database = Database::new_from_url(&database_url).await;
    database.run_migrations().await.expect("Migrations failed");

    let pool = database.pool();

    let result = sqlx::query("INSERT INTO channels (id) VALUES ($1)")
        .bind("test1234")
        .execute(pool)
        .await;

    assert!(
        result.is_err(),
        "Insert without required fields should fail"
    );

    let result = sqlx::query(
        r#"INSERT INTO channels (id, name, creator_ip)
           VALUES ($1, $2, $3)"#,
    )
    .bind("test1234")
    .bind("Test Channel")
    .bind("127.0.0.1")
    .execute(pool)
    .await;

    assert!(result.is_ok(), "Insert with required fields should succeed");
}
