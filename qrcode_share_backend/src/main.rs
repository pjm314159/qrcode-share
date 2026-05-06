//! QRcode Share Backend Server Entry Point
//!
//! This is the main entry point for the QRcode Share backend server.

use std::net::SocketAddr;
use std::sync::Arc;

use qrcode_share_backend::{build_router, start_cleanup_task, start_wechat_refresh_task, AppState, Config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the tracing subscriber for structured logging
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,qrcode_share_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

/// Create an optimized TCP listener using socket2
fn create_optimized_listener(addr: SocketAddr) -> std::net::TcpListener {
    use socket2::{Domain, Protocol, Socket, Type};

    let domain = if addr.is_ipv6() {
        Domain::IPV6
    } else {
        Domain::IPV4
    };

    let socket =
        Socket::new(domain, Type::STREAM, Some(Protocol::TCP)).expect("Failed to create socket");

    // Allow quick restart after crash
    socket
        .set_reuse_address(true)
        .expect("Failed to set SO_REUSEADDR");

    // Disable Nagle's algorithm for low-latency WebSocket
    socket.set_nodelay(true).expect("Failed to set TCP_NODELAY");

    // Set keepalive for connection health
    socket.set_keepalive(true).expect("Failed to set keepalive");

    // Bind and listen with large backlog
    socket.bind(&addr.into()).expect("Failed to bind socket");
    socket.listen(1024).expect("Failed to listen on socket");

    socket.into()
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    init_tracing();

    // Load and validate configuration
    let config = Config::from_env()?;
    config.validate()?;

    tracing::info!(
        host = %config.host,
        port = config.port,
        max_channels = config.max_channels,
        max_messages_per_channel = config.max_messages_per_channel,
        "Starting QRcode Share server"
    );

    // Create application state
    let config = Arc::new(config);
    let app_state = AppState::new(config.clone());

    // Start background cleanup task
    let cleanup_handle = start_cleanup_task(app_state.clone());
    tracing::info!("Background cleanup task started");

    // Start WeChat token refresh task
    let wechat_refresh_handle = start_wechat_refresh_task(app_state.clone());
    tracing::info!("WeChat token refresh task started");

    // Verify WeChat JS-SDK configuration at startup
    app_state.verify_wechat_config().await;

    // Build the router
    let app = build_router(app_state);

    // Create optimized TCP listener
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = create_optimized_listener(addr);
    let listener = tokio::net::TcpListener::from_std(listener)?;

    tracing::info!("Server listening on {}", addr);

    // Create shutdown broadcast channel
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

    // Spawn shutdown signal handler
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        let _ = shutdown_tx_clone.send(());
    });

    // Start the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_rx.recv().await.ok();
        })
        .await?;

    // Cleanup: abort background tasks
    cleanup_handle.abort();
    wechat_refresh_handle.abort();

    tracing::info!("Server shutdown complete");
    Ok(())
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received");
}
