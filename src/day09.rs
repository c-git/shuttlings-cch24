use actix_web::{web, HttpResponse};
use tracing::instrument;

#[instrument]
async fn milk() -> actix_web::Result<HttpResponse> {
    todo!()
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/9").route("/milk", web::post().to(milk))
}
