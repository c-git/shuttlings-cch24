use actix_web::{
    middleware::Logger,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};

pub fn modify_service_config(cfg: &mut ServiceConfig) {
    cfg.default_service(web::route().to(not_found).wrap(Logger::default()));
}

#[tracing::instrument(ret, level = "error")]
pub async fn not_found(req: HttpRequest) -> HttpResponse {
    HttpResponse::NotFound().body("404 - Not found\n")
}
