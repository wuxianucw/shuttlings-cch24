mod c2;

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
        .nest("/2", c2::router());

    Ok(router.into())
}
