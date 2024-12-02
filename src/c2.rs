use std::net::{Ipv4Addr, Ipv6Addr};

use axum::{extract::Query, routing::get, Router};
use itertools::Itertools;
use serde::Deserialize;

#[derive(Deserialize)]
struct DestParam<T> {
    from: T,
    key: T,
}

#[derive(Deserialize)]
struct KeyParam<T> {
    from: T,
    to: T,
}

async fn v4_dest(Query(param): Query<DestParam<Ipv4Addr>>) -> String {
    param
        .from
        .octets()
        .into_iter()
        .zip(param.key.octets().into_iter())
        .map(|(from, key)| from.overflowing_add(key).0)
        .join(".")
}

async fn v4_key(Query(param): Query<KeyParam<Ipv4Addr>>) -> String {
    param
        .from
        .octets()
        .into_iter()
        .zip(param.to.octets().into_iter())
        .map(|(from, to)| to.overflowing_sub(from).0)
        .join(".")
}

async fn v6_dest(Query(param): Query<DestParam<Ipv6Addr>>) -> String {
    Ipv6Addr::from_bits(
        param
            .from
            .octets()
            .into_iter()
            .zip(param.key.octets().into_iter())
            .map(|(from, key)| from ^ key)
            .fold(0u128, |acc, x| (acc << 8) | (x as u128)),
    )
    .to_string()
}

async fn v6_key(Query(param): Query<KeyParam<Ipv6Addr>>) -> String {
    //    to = from ^ key
    // => from ^ to = from ^ from ^ key = key
    // => key = from ^ to
    Ipv6Addr::from_bits(
        param
            .from
            .octets()
            .into_iter()
            .zip(param.to.octets().into_iter())
            .map(|(from, key)| from ^ key)
            .fold(0u128, |acc, x| (acc << 8) | (x as u128)),
    )
    .to_string()
}

#[inline(always)]
pub fn router() -> Router {
    Router::new()
        .route("/dest", get(v4_dest))
        .route("/key", get(v4_key))
        .route("/v6/dest", get(v6_dest))
        .route("/v6/key", get(v6_key))
}
