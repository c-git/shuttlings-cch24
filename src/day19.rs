use actix_web::{error, web, HttpResponse};
use anyhow::Context;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Quote {
    id: Uuid,
    author: String,
    quote: String,
    created_at: chrono::DateTime<Utc>,
    version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct DraftQuote {
    author: String,
    quote: String,
}

#[instrument(ret, err(Debug), skip(pool))]
async fn reset(pool: web::Data<PgPool>) -> actix_web::Result<HttpResponse> {
    let pool: &PgPool = &pool;
    sqlx::query!("TRUNCATE quotes;")
        .execute(pool)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}

#[instrument(ret, err(Debug), skip(pool))]
async fn get_quote(pool: &PgPool, id: &Uuid) -> actix_web::Result<Quote> {
    sqlx::query_as!(
        Quote,
        "SELECT id, author, quote, created_at, version FROM quotes where id = $1;",
        id
    )
    .fetch_optional(pool)
    .await
    .map_err(error::ErrorInternalServerError)?
    .context("id not found")
    .map_err(error::ErrorNotFound)
}

#[instrument(ret, err(Debug), skip(pool))]
async fn cite(path: web::Path<String>, pool: web::Data<PgPool>) -> actix_web::Result<HttpResponse> {
    let pool: &PgPool = &pool;
    let id = Uuid::try_parse(&path.into_inner()).map_err(error::ErrorBadRequest)?;
    let quote = get_quote(pool, &id).await?;
    Ok(HttpResponse::Ok().json(quote))
}

#[instrument(ret, err(Debug), skip(pool))]
async fn remove(
    path: web::Path<String>,
    pool: web::Data<PgPool>,
) -> actix_web::Result<HttpResponse> {
    let pool: &PgPool = &pool;
    let id = Uuid::try_parse(&path.into_inner()).map_err(error::ErrorBadRequest)?;
    let quote = get_quote(pool, &id).await?;
    sqlx::query!("DELETE FROM quotes WHERE id = $1", id)
        .execute(pool)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(quote))
}

#[instrument(ret, err(Debug), skip(pool))]
async fn undo(
    web::Json(draft): web::Json<DraftQuote>,
    path: web::Path<String>,
    pool: web::Data<PgPool>,
) -> actix_web::Result<HttpResponse> {
    let pool: &PgPool = &pool;
    let id = Uuid::try_parse(&path.into_inner()).map_err(error::ErrorBadRequest)?;
    let query_result = sqlx::query!(
        "UPDATE quotes 
        SET
        author = $1, 
        quote = $2,
        version = version + 1
        WHERE id = $3;",
        draft.author,
        draft.quote,
        id
    )
    .execute(pool)
    .await
    .map_err(error::ErrorInternalServerError)?;
    if query_result.rows_affected() != 1 {
        return Err(error::ErrorNotFound(format!(
            "Expected 1 row to match but found: {}",
            query_result.rows_affected()
        )));
    }
    let quote = get_quote(pool, &id).await?;
    Ok(HttpResponse::Ok().json(quote))
}

#[instrument(ret, err(Debug), skip(pool))]
async fn draft(
    web::Json(draft): web::Json<DraftQuote>,
    pool: web::Data<PgPool>,
) -> actix_web::Result<HttpResponse> {
    let pool: &PgPool = &pool;
    let id = Uuid::new_v4();
    let quote = sqlx::query_as!(
        Quote,
        "INSERT INTO quotes 
        (id, author, quote) 
        values ($1, $2, $3) 
        RETURNING id, author, quote, created_at, version;",
        id,
        draft.author,
        draft.quote
    )
    .fetch_one(pool)
    .await
    .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Created().json(quote))
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/19")
        .route("/reset", web::post().to(reset))
        .route("/cite/{id}", web::get().to(cite))
        .route("/remove/{id}", web::delete().to(remove))
        .route("/undo/{id}", web::put().to(undo))
        .route("/draft", web::post().to(draft))
}
