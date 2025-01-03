use actix_files::Files;
use actix_web::{
    middleware::Logger,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};

mod day02;
mod day05;
mod day09;
mod day12;
mod day16;
mod day19;
mod day23;
mod day_minus_1;

/// This function is called once per worker
fn modify_service_config(cfg: &mut ServiceConfig) {
    cfg.route("/", web::get().to(day_minus_1::task1));
    cfg.service(day_minus_1::scope().wrap(Logger::default()));
    cfg.service(day02::scope().wrap(Logger::default()));
    cfg.service(day05::scope().wrap(Logger::default()));
    cfg.service(day09::scope().wrap(Logger::default()));
    cfg.service(day12::scope().wrap(Logger::default()));
    cfg.service(day16::scope().wrap(Logger::default()));
    cfg.service(day19::scope().wrap(Logger::default()));
    cfg.service(day23::scope().wrap(Logger::default()));
    cfg.service(Files::new("/assets", "assets"));
    cfg.default_service(web::route().to(not_found).wrap(Logger::default()));
}

/// This function is called once and returns a closure that is called once per worker
pub fn setup_closure(
    pool: sqlx::PgPool,
) -> impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static {
    // Code that should run exactly once
    let pool = web::Data::new(pool);
    let day09_data = day09::app_data();
    let day12_data = day12::app_data();

    // Closure that is returned
    |cfg: &mut ServiceConfig| {
        cfg.app_data(pool);
        cfg.app_data(day09_data);
        cfg.app_data(day12_data);

        modify_service_config(cfg);
    }
}

#[tracing::instrument(name = "DEFAULT NOT FOUND HANDLER", ret, level = "error")]
pub async fn not_found(req: HttpRequest) -> HttpResponse {
    HttpResponse::NotFound().body("404 - Not found\n")
}
