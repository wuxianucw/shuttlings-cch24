mod c12;
mod c16;
mod c2;
mod c5;
mod c9;

use axum::{
    body::Body,
    http::{header::LOCATION, StatusCode},
    response::Response,
    routing::get,
    Router,
};

async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

async fn seek() -> Response {
    Response::builder()
        .status(StatusCode::FOUND)
        .header(LOCATION, "https://www.youtube.com/watch?v=9Gc4QTqslN4")
        .body(Body::empty())
        .unwrap()
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(hello_bird))
        .route("/-1/seek", get(seek))
        .nest("/2", c2::router())
        .nest("/5", c5::router())
        .nest("/9", c9::router())
        .nest("/12", c12::router())
        .nest("/16", c16::router());

    Ok(router.into())
}
