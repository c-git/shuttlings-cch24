use actix_web::web;
use tracing::instrument;

#[instrument]
async fn star() -> &'static str {
    r#"<div id="star" class="lit"></div>"#
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/23").route("/star", web::get().to(star))
}
