//! Application entry point for the Argus Events server.
use argus_events::{create_metrics, create_repository};
use argus_events::{event_routes, Args};
use clap::Parser;
use tokio::signal;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI args
    let args = Args::parse();

    // Init logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Shared repository
    let repo = create_repository(&args.repository)
        .map_err(|e| anyhow::anyhow!("Failed to create repository: {}", e))?;

    // Route setup
    let metrics = create_metrics()?;
    let app = event_routes(repo, metrics);

    // Launch server
    let listener = tokio::net::TcpListener::bind(&args.endpoint).await?;
    tracing::info!("🚀 Server running on http://{}", args.endpoint);

    // Graceful shutdown using tokio::signal::ctrl_c().
    // This was listed as complete in docs/ASSIGNMENT.md, but was not implemented until this commit.
    // Cannot be unit tested directly (due to signal handling), but verified manually:
    // - Server starts
    // - Ctrl+C triggers log and clean shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
            tracing::info!("🛑 Received Ctrl+C, shutting down gracefully...");
        })
        .await?;

    Ok(())
}
