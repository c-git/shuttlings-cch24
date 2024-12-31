use actix_web::{
    middleware::Logger,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};

mod day02;
mod day05;
mod day09;
mod day12;
mod day_minus_1;

/// This function is called once per worker
fn modify_service_config(cfg: &mut ServiceConfig) {
    cfg.route("/", web::get().to(day_minus_1::task1));
    cfg.service(day_minus_1::scope().wrap(Logger::default()));
    cfg.service(day02::scope().wrap(Logger::default()));
    cfg.service(day05::scope().wrap(Logger::default()));
    cfg.service(day09::scope().wrap(Logger::default()));
    cfg.service(day12::scope().wrap(Logger::default()));
    cfg.default_service(web::route().to(not_found).wrap(Logger::default()));
}

/// This function is called once and returns a closure that is called once per worker
pub fn setup_closure() -> impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static {
    // Code that should run exactly once
    let day09_data = day09::app_data();
    let day12_data = day12::app_data();

    // Closure that is returned
    |cfg: &mut ServiceConfig| {
        cfg.app_data(day09_data);
        cfg.app_data(day12_data);

        modify_service_config(cfg);
    }
}

#[tracing::instrument(ret, level = "error")]
pub async fn not_found(req: HttpRequest) -> HttpResponse {
    HttpResponse::NotFound().body("404 - Not found\n")
}
