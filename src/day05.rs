use actix_web::{error, web, HttpResponse};
use anyhow::Context;
use cargo_manifest::{Manifest, MaybeInherited};
use std::fmt::{Debug, Display};
use toml::Value;
use tracing::{instrument, warn};

#[derive(Debug)]
struct Order {
    item: String,
    quantity: u32,
}

#[instrument(ret, err)]
async fn manifest(data: String) -> actix_web::Result<HttpResponse> {
    let mut result: Vec<Order> = vec![];
    let manifest: Manifest<Value> =
        Manifest::from_slice_with_metadata(data.as_bytes()).map_err(bad_request)?;
    let package = manifest
        .package
        .context("no package section")
        .map_err(bad_request)?;
    match package.keywords {
        Some(MaybeInherited::Local(words)) if words.iter().any(|word| word == "Christmas 2024") => {
        }
        _ => return Err(error::ErrorBadRequest("Magic keyword not provided")),
    }
    let Some(metadata) = package.metadata else {
        return no_content();
    };
    let Some(orders) = metadata
        .as_table()
        .context("not a table")
        .map_err(bad_request)?
        .get("orders")
    else {
        return no_content();
    };
    for raw_order in orders
        .as_array()
        .context("orders is not an array")
        .map_err(bad_request)?
        .iter()
    {
        if let Some(order) = extract_order(raw_order) {
            result.push(order);
        }
    }
    if result.is_empty() {
        no_content()
    } else {
        Ok(HttpResponse::Ok().body(
            result
                .into_iter()
                .map(|order| order.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        ))
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.item, self.quantity)
    }
}

fn extract_order(raw_order: &Value) -> Option<Order> {
    let order = raw_order.as_table()?;
    let item = order.get("item")?.as_str()?.to_string();
    let Ok(quantity) = order.get("quantity")?.as_integer()?.try_into() else {
        return None;
    };
    Some(Order { item, quantity })
}

fn no_content() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::NoContent().finish())
}

fn bad_request<T: Debug>(err: T) -> actix_web::Error {
    warn!(?err);
    error::ErrorBadRequest("Invalid manifest")
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/5").route("/manifest", web::post().to(manifest))
}
