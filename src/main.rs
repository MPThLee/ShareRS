use std::sync::Arc;

use axum::{routing::get, Extension, Router};
use routes::get_hash;

use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;
mod storage;

#[tokio::main]
async fn main() {
    // Initialize dotenv
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sharers=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(true).compact())
        .init();

    let storage_backend =
        dotenvy::var("SHARERS_STORAGE_BACKEND").unwrap_or_else(|_| "local".to_string());
    let storage: Arc<dyn storage::Storage + Send + Sync> = match storage_backend.as_str() {
        "s3" => Arc::new(
            storage::S3::new(
                &dotenvy::var("SHARERS_S3_BUCKET_NAME").unwrap(),
                &dotenvy::var("SHARERS_S3_REGION").unwrap(),
                &dotenvy::var("SHARERS_S3_URL").unwrap(),
                &dotenvy::var("SHARERS_S3_ACCESS").unwrap(),
                &dotenvy::var("SHARERS_S3_SECRET").unwrap(),
            )
            .unwrap(),
        ),
        "local" => Arc::new(storage::Local::new(
            dotenvy::var("SHARERS_LOCAL_FILE_PATH").unwrap(),
        )),
        _ => panic!("Invalid storage backend specified. Aborting startup!"),
    };

    let app = Router::new()
        .route("/*hash", get(get_hash))
        .layer(Extension(storage))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = dotenvy::var("SHARERS_BIND_ADDR").unwrap();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
