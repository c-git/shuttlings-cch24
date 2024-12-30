use actix_web::{web, HttpResponse};

#[instrument]
async fn milk() -> actix_web::Result<HttpResponse> {
    todo!()
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/16").route("/milk", web::post().to(milk))
}
