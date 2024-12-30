use actix_web::{
    error,
    http::header::ContentType,
    middleware::Logger,
    web::{self, ServiceConfig},
    HttpResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fmt::Display, sync::Mutex, time::Instant};
use tracing::{info, instrument, warn};

#[derive(Debug)]
struct Bucket {
    remainder: u8,
    last_wd: Instant,
}

#[derive(Debug)]
enum WithdrawOutcome {
    MilkWithdrawn,
    NoMilkLeft,
}

impl Bucket {
    const CAPACITY: u8 = 5;
    fn new() -> Self {
        Self {
            remainder: Self::CAPACITY,
            last_wd: Instant::now(),
        }
    }

    fn new_wrapped() -> web::Data<Mutex<Self>> {
        web::Data::new(Mutex::new(Self::new()))
    }

    fn try_withdraw(&mut self) -> WithdrawOutcome {
        self.refill_by_time();
        if self.remainder > 0 {
            self.remainder -= 1;
            WithdrawOutcome::MilkWithdrawn
        } else {
            WithdrawOutcome::NoMilkLeft
        }
    }

    fn refill_to_max(&mut self) {
        self.last_wd = Instant::now();
        self.remainder = Self::CAPACITY;
    }

    fn refill_by_time(&mut self) {
        let secs = Instant::now().duration_since(self.last_wd).as_secs();
        if secs > 0 {
            self.remainder = self
                .remainder
                .saturating_add(secs.try_into().unwrap_or(Self::CAPACITY))
                .min(Self::CAPACITY);
            self.last_wd = Instant::now();
        }
    }
}

impl Display for WithdrawOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            match self {
                WithdrawOutcome::MilkWithdrawn => "Milk withdrawn",
                WithdrawOutcome::NoMilkLeft => "No milk available",
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConversionRequest {
    liters: Option<f32>,
    gallons: Option<f32>,
    litres: Option<f32>,
    pints: Option<f32>,
}

#[instrument]
async fn milk(
    data: String,
    content_type: Option<web::Header<ContentType>>,
    bucket: web::Data<Mutex<Bucket>>,
) -> actix_web::Result<HttpResponse> {
    const LITERS_TO_US_GAL: f32 = 3.7854118;
    const LITERS_TO_UK_PINT: f32 = 0.568261;
    let mut converted = None;

    let outcome = bucket.lock().unwrap().try_withdraw();

    if Some(web::Header(ContentType::json())) == content_type {
        let conversion_request: ConversionRequest =
            serde_json::from_str(&data).map_err(error::ErrorBadRequest)?;
        converted = match conversion_request {
            ConversionRequest {
                liters: Some(value),
                gallons: None,
                litres: None,
                pints: None,
            } => Some(json!({"gallons": value / LITERS_TO_US_GAL })),
            ConversionRequest {
                liters: None,
                gallons: Some(value),
                litres: None,
                pints: None,
            } => Some(json!({"liters": value * LITERS_TO_US_GAL })),
            ConversionRequest {
                liters: None,
                gallons: None,
                litres: Some(value),
                pints: None,
            } => Some(json!({"pints": value / LITERS_TO_UK_PINT })),
            ConversionRequest {
                liters: None,
                gallons: None,
                litres: None,
                pints: Some(value),
            } => Some(json!({"litres": value * LITERS_TO_UK_PINT })),
            _ => {
                warn!(?conversion_request, "Invalid conversion request");
                return Err(error::ErrorBadRequest(""));
            }
        };
    }

    info!(?outcome, ?converted);

    match &outcome {
        WithdrawOutcome::MilkWithdrawn => {
            if let Some(converted) = converted {
                Ok(HttpResponse::Ok().json(converted))
            } else {
                Ok(HttpResponse::Ok().body(outcome.to_string()))
            }
        }
        WithdrawOutcome::NoMilkLeft => Err(error::ErrorTooManyRequests(outcome.to_string())),
    }
}

pub fn add_day09(cfg: &mut ServiceConfig) {
    cfg.service(scope().wrap(Logger::default()));

    // This not correct but it is simple
    // Each thread will create it's own bucket but there will not be duplicates
    // Each new one created will replace the previous one so there will only be one.
    // It results in a few extra allocations but I think for the code simplicity it's still a win
    // See quote from actix_web docs
    //
    // > If an item of this type was already stored, it will be replaced and returned.
    cfg.app_data(Bucket::new_wrapped());
}

#[instrument]
async fn refill(bucket: web::Data<Mutex<Bucket>>) -> HttpResponse {
    bucket.lock().unwrap().refill_to_max();
    HttpResponse::Ok().finish()
}

fn scope() -> actix_web::Scope {
    web::scope("/9")
        .route("/milk", web::post().to(milk))
        .route("/refill", web::post().to(refill))
}
