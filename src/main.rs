use std::sync::Arc;

use actix_web::{web, App, HttpServer};

mod routes;
mod storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

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
            &dotenvy::var("SHARERS_LOCAL_FILE_PATH").unwrap(),
        )),
        _ => panic!("Invalid storage backend specified. Aborting startup!"),
    };

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Compress::default())
            .app_data(web::Data::new(storage.clone()))
            .default_service(web::get().to(routes::get_hash))
    })
    .bind(dotenvy::var("SHARERS_BIND_ADDR").unwrap())?
    .run()
    .await
}
