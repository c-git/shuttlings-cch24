use actix_multipart::Multipart;
use actix_web::{error, web};
use anyhow::{bail, Context};
use cargo_lock::Lockfile;
use futures_util::StreamExt as _;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use tracing::{debug, info, instrument};

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

#[instrument(skip_all, ret, err(Debug))]
async fn lockfile(payload: Multipart) -> actix_web::Result<String> {
    let lockfile_contents = get_lockfile_contents(payload)
        .await
        .map_err(error::ErrorBadRequest)?;
    debug!(lockfile_contents);

    let mut checksums = vec![];
    println!("Lockfile Contents:\n{lockfile_contents}"); // TODO remove test code
    let lockfile: Lockfile = lockfile_contents
        .parse()
        .context("failed to parse lockfile contents")
        .map_err(error::ErrorBadRequest)?;
    lockfile
        .packages
        .into_iter()
        .filter_map(|x| x.checksum)
        .for_each(|x| checksums.push(x.to_string()));

    Ok(checksums
        .into_iter()
        .map(checksum_to_style)
        .collect::<Result<Vec<_>, _>>()
        .map_err(error::ErrorUnprocessableEntity)?
        .join("\n"))
}

async fn get_lockfile_contents(mut payload: Multipart) -> anyhow::Result<String> {
    let mut bytes = vec![];
    // iterate over multipart stream
    while let Some(item) = payload.next().await {
        let Ok(mut field) = item else {
            bail!("failed to get payload field");
        };
        let is_correct_field = field.name() == Some("lockfile");

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            if is_correct_field {
                let Ok(chunk) = chunk else {
                    bail!("failed to get a chunk of the payload");
                };
                chunk.into_iter().for_each(|b| bytes.push(b));
            }
        }
    }

    let result = std::str::from_utf8(&bytes).context("payload was invalid utf8")?;
    Ok(result.to_string())
}

#[instrument(ret, err(Debug))]
fn checksum_to_style<S: AsRef<str> + Debug>(check_sum: S) -> anyhow::Result<String> {
    let check_sum = check_sum.as_ref().as_bytes();
    if check_sum.len() < 10 {
        bail!("checksum is less than 10 bytes long");
    }
    let color = std::str::from_utf8(&check_sum[..6]).context("failed to get color")?;
    if !color.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("Invalid character found in color");
    }
    let top_str = std::str::from_utf8(&check_sum[6..8]).context("failed to get top")?;
    let top = u8::from_str_radix(top_str, 16).context("failed to parse top")?;
    let left_str = std::str::from_utf8(&check_sum[8..10]).context("failed to get left")?;
    let left = u8::from_str_radix(left_str, 16).context("failed to parse left")?;
    info!(color, top_str, left_str);
    Ok(format!(
        r#"<div style="background-color:#{color};top:{top}px;left:{left}px;"></div>"#
    ))
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/23")
        .route("/star", web::get().to(star))
        .route("/present/{color}", web::get().to(present))
        .route("/ornament/{state}/{n}", web::get().to(ornament))
        .route("/lockfile", web::post().to(lockfile))
}
