use std::sync::Arc;

use anyhow::Context;
use axum::{routing::get, Extension, Router};
use routes::get_hash;

use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod database;
mod routes;
mod storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize dotenv
    dotenvy::dotenv().ok();

    // Logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sharers=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(true).compact())
        .init();

    // Database
    database::check_for_migrations()
        .await
        .expect("An error occurred while running migrations.");

    let pool = database::connect()
        .await
        .expect("Database connection failed");

    // Storage
    let storage_backend = dotenvy::var("STORAGE_BACKEND").unwrap_or_else(|_| "local".to_string());
    let storage: Arc<dyn storage::Storage + Send + Sync> = match storage_backend.as_str() {
        "s3" => Arc::new(storage::S3::new(
            &dotenvy::var("S3_BUCKET_NAME")?,
            &dotenvy::var("S3_REGION")?,
            &dotenvy::var("S3_URL")?,
            &dotenvy::var("S3_ACCESS")?,
            &dotenvy::var("S3_SECRET")?,
        )?),
        "local" => Arc::new(storage::Local::new(dotenvy::var("LOCAL_FILE_PATH")?)),
        _ => panic!("Invalid storage backend specified. Aborting startup!"),
    };

    // App handler
    let app = Router::new()
        .route("/*hash", get(get_hash))
        .layer(Extension(storage))
        .layer(Extension(pool))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = dotenvy::var("BIND_ADDR")?;
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await
        .context("Failed to serve service")
}
