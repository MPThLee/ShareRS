use actix_web::{HttpResponse, Responder};

pub async fn get_hash() -> impl Responder {
    HttpResponse::NotFound().body("Unimplemented")
}