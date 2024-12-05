use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    routing::post,
    Router,
};
use cargo_manifest::Manifest;
use itertools::Itertools;
use toml::Table;

fn extract_item(x: &Table) -> Option<(&str, i64)> {
    let item = x.get("item")?.as_str()?;
    let quantity = x.get("quantity")?.as_integer()?;
    Some((item, quantity))
}

async fn manifest(
    headers: HeaderMap,
    body: String,
) -> Result<(StatusCode, String), (StatusCode, &'static str)> {
    let content_type = headers.get(CONTENT_TYPE).and_then(|v| v.to_str().ok());
    let manifest = match content_type {
        Some("application/toml") => toml::from_str::<Manifest>(&body).ok(),
        Some("application/yaml") => serde_yml::from_str(&body).ok(),
        Some("application/json") => serde_json::from_str(&body).ok(),
        _ => Err((StatusCode::UNSUPPORTED_MEDIA_TYPE, ""))?,
    }
    .ok_or((StatusCode::BAD_REQUEST, "Invalid manifest"))?;

    let package = manifest
        .package
        .ok_or((StatusCode::BAD_REQUEST, "Magic keyword not provided"))?;
    if !package
        .keywords
        .and_then(|k| k.as_local())
        .is_some_and(|k| k.iter().any(|x| x == "Christmas 2024"))
    {
        Err((StatusCode::BAD_REQUEST, "Magic keyword not provided"))?;
    }

    let orders = package
        .metadata
        .as_ref()
        .and_then(|m| m.get("orders"))
        .and_then(|o| o.as_array())
        .ok_or((StatusCode::NO_CONTENT, ""))?;
    let plain_list = orders
        .iter()
        .filter_map(|x| x.as_table())
        .filter_map(extract_item)
        .map(|(k, v)| format!("{k}: {v}"))
        .join("\n");

    if plain_list.is_empty() {
        Err((StatusCode::NO_CONTENT, ""))
    } else {
        Ok((StatusCode::OK, plain_list))
    }
}

#[inline(always)]
pub fn router() -> Router {
    Router::new().route("/manifest", post(manifest))
}
