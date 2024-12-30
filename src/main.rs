use actix_web::web::ServiceConfig;
use shuttle_actix_web::ShuttleActixWeb;

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    use tracing_subscriber::{
        fmt::{self, format::FmtSpan},
        prelude::*,
        EnvFilter,
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_span_events(FmtSpan::NEW))
        // .with(fmt::layer().with_span_events(FmtSpan::ACTIVE))
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    Ok(shuttlings_cch24::modify_service_config.into())
}
