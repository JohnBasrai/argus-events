//! Application entry point for the Argus Events server.

use argus_events::create_repository;
use argus_events::{event_routes, Args};
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI args
    let args = Args::parse();

    // Init logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Shared repository
    let repo = create_repository(&args.repository)
        .map_err(|e| anyhow::anyhow!("Failed to create repository: {}", e))?;

    // Route setup
    let app = event_routes(repo);

    // Launch server
    let listener = tokio::net::TcpListener::bind(&args.endpoint).await?;

    tracing::info!("🚀 Server running on http://{}", args.endpoint);

    // Use axum::serve instead of hyper::Server
    axum::serve(listener, app).await?;

    Ok(())
}