use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use leaky_bucket::RateLimiter;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Amount {
    Liters(f32),
    Gallons(f32),
    Litres(f32),
    Pints(f32),
}

fn transform_amount(amount: Amount) -> Amount {
    match amount {
        Amount::Liters(x) => Amount::Gallons(x * 0.264172),
        Amount::Gallons(x) => Amount::Liters(x * 3.78541),
        Amount::Litres(x) => Amount::Pints(x * 1.759754),
        Amount::Pints(x) => Amount::Litres(x * 0.568261),
    }
}

type RateLimitError = (StatusCode, &'static str);

fn acquire_milk(limiter: Arc<Mutex<RateLimiter>>) -> Result<(), RateLimitError> {
    let limiter = limiter.lock().unwrap();
    if limiter.try_acquire(1) {
        Ok(())
    } else {
        Err((StatusCode::TOO_MANY_REQUESTS, "No milk available\n"))
    }
}

async fn milk(
    State(limiter): State<Arc<Mutex<RateLimiter>>>,
    headers: HeaderMap,
    payload: Option<Json<Amount>>,
) -> Result<Response, RateLimitError> {
    acquire_milk(limiter)?;

    if let Some(Json(payload)) = payload {
        return Ok(Json(transform_amount(payload)).into_response());
    }
    if let Some("application/json") = headers.get(CONTENT_TYPE).and_then(|v| v.to_str().ok()) {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    }

    Ok((StatusCode::OK, "Milk withdrawn\n").into_response())
}

async fn refill(State(limiter): State<Arc<Mutex<RateLimiter>>>) {
    let mut limiter = limiter.lock().unwrap();
    *limiter = new_rate_limiter();
}

#[inline(always)]
fn new_rate_limiter() -> RateLimiter {
    RateLimiter::builder()
        .interval(Duration::from_secs(1))
        .initial(5)
        .max(5)
        .build()
}

#[inline(always)]
pub fn router() -> Router {
    Router::new()
        .route("/milk", post(milk))
        .route("/refill", post(refill))
        .with_state(Arc::new(Mutex::new(new_rate_limiter())))
}
