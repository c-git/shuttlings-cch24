use std::{fmt::Display, str::FromStr};

use actix_web::{error, web};
use tracing::instrument;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Color {
    Red,
    Blue,
    Purple,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum OrnamentState {
    On,
    Off,
}

impl FromStr for OrnamentState {
    type Err = actix_web::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "on" => Self::On,
            "off" => Self::Off,
            other => return Err(error::ErrorImATeapot(format!("Unexpected state: {other}"))),
        })
    }
}

impl Display for OrnamentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::On => "on",
                Self::Off => "off",
            }
        )
    }
}

impl OrnamentState {
    fn toggle(&self) -> Self {
        match self {
            OrnamentState::On => Self::Off,
            OrnamentState::Off => Self::On,
        }
    }

    /// Needs to have a leading space for separation
    fn as_class_str(&self) -> &'static str {
        if self == &Self::On {
            " on"
        } else {
            ""
        }
    }
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

#[instrument(ret, err(Debug))]
async fn ornament(path: web::Path<(String, String)>) -> actix_web::Result<String> {
    let (state, n) = path.into_inner();
    let n = tera::escape_html(&n);
    let state: OrnamentState = state.parse()?;
    let next_state = state.toggle();
    Ok(format!(
        r#"<div class="ornament{}" id="ornament{n}" hx-trigger="load delay:2s once" hx-get="/23/ornament/{next_state}/{n}" hx-swap="outerHTML"></div>"#,
        state.as_class_str()
    ))
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/23")
        .route("/star", web::get().to(star))
        .route("/present/{color}", web::get().to(present))
        .route("/ornament/{state}/{n}", web::get().to(ornament))
}
