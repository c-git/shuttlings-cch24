use std::sync::LazyLock;

use actix_web::{cookie::Cookie, error, web, HttpRequest, HttpResponse};
use anyhow::Context as _;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::{distributions::Alphanumeric, thread_rng, Rng as _};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

#[derive(Debug, Serialize, Deserialize)]
struct GiftWrapper {
    client_gift: serde_json::Value,
    exp: u64,
}
impl GiftWrapper {
    fn new(client_gift: serde_json::Value) -> Self {
        Self {
            client_gift,
            exp: 10000000000,
        }
    }
}

fn encoding_key() -> &'static EncodingKey {
    keys().0
}
fn decoding_key() -> &'static DecodingKey {
    keys().1
}
fn keys() -> (&'static EncodingKey, &'static DecodingKey) {
    static LAZY_LOCK: LazyLock<(EncodingKey, DecodingKey)> = LazyLock::new(|| {
        info!("JWT Keys generated");
        let key: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        (
            EncodingKey::from_secret(key.as_bytes()),
            DecodingKey::from_secret(key.as_bytes()),
        )
    });
    (&LAZY_LOCK.0, &LAZY_LOCK.1)
}

#[instrument(ret, err)]
async fn wrap(
    web::Json(client_gift): web::Json<serde_json::Value>,
) -> actix_web::Result<HttpResponse> {
    let my_claims = GiftWrapper::new(client_gift);
    let token = encode(&Header::default(), &my_claims, encoding_key())
        .map_err(error::ErrorInternalServerError)?;
    let cookie = Cookie::new("gift", token);
    Ok(HttpResponse::Ok().cookie(cookie).finish())
}

#[instrument(ret, err(Debug))]
async fn unwrap(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let cookie = req
        .cookie("gift")
        .context("gift cookie not found")
        .map_err(error::ErrorBadRequest)?;
    let token = cookie.value();
    let token_data =
        decode::<GiftWrapper>(token, decoding_key(), &Validation::new(Algorithm::HS256))
            .context("failed to decode token")
            .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(token_data.claims.client_gift))
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/16")
        .route("/wrap", web::post().to(wrap))
        .route("/unwrap", web::get().to(unwrap))
}
