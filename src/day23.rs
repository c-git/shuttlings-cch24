use std::{fmt::Display, str::FromStr};

use actix_web::{error, web};
use tracing::instrument;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Color {
    Red,
    Blue,
    Purple,
}

impl FromStr for Color {
    type Err = actix_web::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "red" => Self::Red,
            "blue" => Self::Blue,
            "purple" => Self::Purple,
            other => return Err(error::ErrorImATeapot(format!("Unexpected color: {other}"))),
        })
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Color::Red => "red",
                Color::Blue => "blue",
                Color::Purple => "purple",
            }
        )
    }
}

impl Color {
    fn next(&self) -> Self {
        match self {
            Color::Red => Color::Blue,
            Color::Blue => Color::Purple,
            Color::Purple => Color::Red,
        }
    }
}

#[instrument]
async fn star() -> &'static str {
    r#"<div id="star" class="lit"></div>"#
}

#[instrument(ret, err(Debug))]
async fn present(path: web::Path<String>) -> actix_web::Result<String> {
    let requested_color: Color = path.into_inner().parse()?;
    let next_color = requested_color.next();
    Ok(format!(
        r#"
                <div class="present {requested_color}" hx-get="/23/present/{next_color}" hx-swap="outerHTML">
                    <div class="ribbon"></div>
                    <div class="ribbon"></div>
                    <div class="ribbon"></div>
                    <div class="ribbon"></div>
                </div>"#
    ))
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/23")
        .route("/star", web::get().to(star))
        .route("/present/{color}", web::get().to(present))
}
