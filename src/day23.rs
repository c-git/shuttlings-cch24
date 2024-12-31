use actix_web::{web, HttpResponse};
use tracing::instrument;

#[instrument]
async fn star() -> actix_web::Result<HttpResponse> {
    todo!()
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/23").route("/star", web::get().to(star))
}
