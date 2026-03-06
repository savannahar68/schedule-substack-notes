use std::time::Duration;

use axum::{routing::{delete, get, post}, Router};
use tower_http::cors::CorsLayer;

mod config;
mod db;
mod error;
mod middleware;
mod models;
mod routes;
mod services;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "substack_scheduler=info,tower_http=warn".into()),
        )
        .init();

    let config = config::Config::from_env();

    let pool = db::setup(&config).await.expect("Failed to setup database");
    let key = config.load_or_create_encryption_key();

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    let state = AppState {
        pool: pool.clone(),
        key,
        http_client: http_client.clone(),
    };

    let app = Router::new()
        .route("/api/auth/cookies", post(routes::auth::register_cookies))
        .route("/api/auth/health", get(routes::auth::health))
        .route("/api/notes/schedule", post(routes::notes::schedule))
        .route("/api/notes/queue", get(routes::notes::queue))
        .route("/api/notes/due", get(routes::notes::due))
        .route("/api/notes/:id/result", post(routes::notes::report_result))
        .route("/api/notes/:id", delete(routes::notes::delete))
        .route("/api/notes/:id", axum::routing::put(routes::notes::update))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = config.port;
    let addr = format!("0.0.0.0:{port}");

    println!("╔══════════════════════════════════════════╗");
    println!("║      Substack Notes Scheduler            ║");
    println!("║      Running on http://localhost:{port:<5}   ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    println!("Install the Chrome extension and connect it to this server.");
    println!("Data stored in ./data/");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {addr}"));

    axum::serve(listener, app).await.expect("Server error");
}
