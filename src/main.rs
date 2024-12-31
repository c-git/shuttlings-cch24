use actix_web::web::ServiceConfig;
use shuttle_actix_web::ShuttleActixWeb;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://my_user:secret_pass@localhost:5432/cch24"
    )]
    pool: sqlx::PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    tracing_subscriber::registry()
        .with(fmt::layer().with_span_events(FmtSpan::NEW))
        // .with(fmt::layer().with_span_events(FmtSpan::ACTIVE))
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    Ok(shuttlings_cch24::setup_closure(pool).into())
}
