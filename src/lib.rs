use actix_web::{
    middleware::Logger,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};

mod day02;
mod day_minus_1;

pub fn modify_service_config(cfg: &mut ServiceConfig) {
    cfg.route("/", web::get().to(day_minus_1::task1));
    cfg.service(day_minus_1::scope().wrap(Logger::default()));
    cfg.service(day02::scope().wrap(Logger::default()));
    cfg.default_service(web::route().to(not_found).wrap(Logger::default()));
}

#[tracing::instrument(ret, level = "error")]
pub async fn not_found(req: HttpRequest) -> HttpResponse {
    HttpResponse::NotFound().body("404 - Not found\n")
}
