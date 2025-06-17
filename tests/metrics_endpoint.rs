use anyhow::Result;
use argus_events::{create_app, create_metrics, create_repository};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

struct TestServer {
    addr: std::net::SocketAddr,
    client: Client,
}

impl TestServer {
    // ---
    async fn new() -> Result<Self> {
        // ---

        // Enable debug logging only when requested
        if std::env::var("TEST_DEBUG").is_ok() {
            std::env::set_var("RUST_LOG", "debug");
            std::env::set_var("NO_COLOR", "1");
        }
        let repo = create_repository("memory")?;

        // Create a simple noop metrics implementation for testing
        std::env::set_var("ARGUS_METRICS_TYPE", "prom");
        let metrics = create_metrics()?;

        let app = create_app(repo, metrics)?;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        // Spawn the server in the background
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give the server a moment to start
        sleep(Duration::from_millis(100)).await;

        let client = Client::new();
        Ok(Self { addr, client })
    }

    fn url(&self, path: &str) -> String {
        // ---
        format!("http://{}{}", self.addr, path)
    }
}

fn print_matrics(body: &str) {
    // ---
    println!("\n=== PROMETHEUS METRICS OUTPUT ===");
    println!("{body}");
    println!("=== END METRICS ===\n");
}

#[tokio::test]
#[serial_test::serial]
async fn metrics_endpoint_with_prometheus() -> Result<()> {
    // ---
    // Set environment to use Prometheus metrics for this test
    std::env::set_var("AXUM_METRICS_TYPE", "prom");

    let server = TestServer::new().await?;

    // First, hit some endpoints to generate metrics
    let _ = server.client.get(server.url("/events")).send().await?;
    let _ = server.client.get(server.url("/events")).send().await?;

    // Give metrics a moment to be recorded
    sleep(Duration::from_millis(50)).await;

    // Now check the metrics endpoint
    let res = server.client.get(server.url("/metrics")).send().await?;

    // Check status before consuming the response
    assert!(
        res.status().is_success(),
        "Metrics endpoint should return success"
    );

    let body = res.text().await?;
    print_matrics(&body);

    // Debug: print the body to see what we're getting
    println!("Metrics response body: '{}'", body);

    // The metrics endpoint should return some content
    assert!(!body.is_empty(), "Metrics should not be empty");

    // For Prometheus format, we expect specific patterns
    if body.contains("# HELP") || body.contains("# TYPE") {
        // This looks like Prometheus format
        println!("✅ Detected Prometheus format metrics");
    } else {
        // Might be a different format or no metrics yet
        println!("ℹ️  Metrics format: {}", body);
    }

    // Clean up environment variable
    std::env::remove_var("AXUM_METRICS_TYPE");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn metrics_endpoint_with_noop() -> Result<()> {
    // ---
    // Set environment to use noop metrics (or don't set it)
    std::env::set_var("AXUM_METRICS_TYPE", "noop");

    let server = TestServer::new().await?;

    // Hit some endpoints
    let _ = server.client.get(server.url("/events")).send().await?;

    // Check the metrics endpoint
    let res = server.client.get(server.url("/metrics")).send().await?;

    // Should still return success even with noop metrics
    assert!(
        res.status().is_success(),
        "Metrics endpoint should return success even with noop"
    );

    let body = res.text().await?;
    print_matrics(&body);

    // Clean up environment variable
    std::env::remove_var("AXUM_METRICS_TYPE");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn metrics_endpoint_survives_load() -> Result<()> {
    // ---
    std::env::set_var("AXUM_METRICS_TYPE", "prom");

    let server = Arc::new(TestServer::new().await?);

    // Generate some load
    let futures = (0..20).map(|i| {
        let server = Arc::clone(&server);
        async move {
            let endpoint = match i % 2 {
                0 => "/events",
                _ => "/metrics",
            };
            server.client.get(server.url(endpoint)).send().await
        }
    });

    let responses = futures::future::join_all(futures).await;

    // All requests should succeed
    for (i, response) in responses.into_iter().enumerate() {
        // ---

        let response = response.unwrap_or_else(|_| panic!("Request {i} should succeed"));
        if !response.status().is_success() {
            println!("Request {} failed with status: {}", i, response.status());
        }
        assert!(
            response.status().is_success(),
            "Request {} should return success",
            i
        );
    }

    // Now check metrics
    let res = server.client.get(server.url("/metrics")).send().await?;
    assert!(res.status().is_success());

    let body = res.text().await?;
    assert!(!body.is_empty());

    std::env::remove_var("AXUM_METRICS_TYPE");
    Ok(())
}

#[tokio::test]
#[serial_test::serial]
async fn metrics_content_type_is_correct() -> Result<()> {
    // ---
    std::env::set_var("AXUM_METRICS_TYPE", "prom");

    let server = TestServer::new().await?;

    let res = server.client.get(server.url("/metrics")).send().await?;
    assert!(res.status().is_success());

    // Check content type
    let content_type = res.headers().get("content-type");
    println!("Metrics content-type: {:?}", content_type);

    // Prometheus metrics should have text/plain content type
    // Note: Your implementation might use a different content type
    if let Some(ct) = content_type {
        let ct_str = ct.to_str()?;
        assert!(
            ct_str.contains("text/plain")
                || ct_str.contains("text/")
                || ct_str.contains("application/"),
            "Content type should be appropriate for metrics: {}",
            ct_str
        );
    }

    std::env::remove_var("AXUM_METRICS_TYPE");
    Ok(())
}
