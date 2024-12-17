use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use jsonwebtoken::{
    errors::ErrorKind as JwtErrorKind, Algorithm, DecodingKey, EncodingKey, Header as JwtHeader,
    Validation,
};
use serde_json::Value;

const SECRET: &str = "__ucw__";
const SANTA_PUBKEY: &[u8] = include_bytes!("day16_santa_public_key.pem");

async fn wrap(jar: CookieJar, Json(payload): Json<Value>) -> Result<impl IntoResponse, StatusCode> {
    let token = jsonwebtoken::encode(
        &JwtHeader::default(),
        &payload,
        &EncodingKey::from_secret(SECRET.as_bytes()),
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(jar.add(Cookie::new("gift", token)))
}

async fn unwrap(jar: CookieJar) -> Result<impl IntoResponse, StatusCode> {
    let token = jar
        .get("gift")
        .ok_or(StatusCode::BAD_REQUEST)?
        .value_trimmed();
    let mut validation = Validation::default();
    validation.set_required_spec_claims::<&str>(&[]);
    validation.validate_exp = false;
    let data = jsonwebtoken::decode::<Value>(
        token,
        &DecodingKey::from_secret(SECRET.as_bytes()),
        &validation,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(data.claims))
}

fn try_algorithms(token: &str, key: &DecodingKey) -> Result<Value, JwtErrorKind> {
    const ALL_ALGORITHMS: [Algorithm; 11] = [
        Algorithm::ES256,
        Algorithm::ES384,
        Algorithm::EdDSA,
        Algorithm::HS384,
        Algorithm::HS512,
        Algorithm::PS256,
        Algorithm::PS384,
        Algorithm::PS512,
        Algorithm::RS256,
        Algorithm::RS384,
        Algorithm::RS512,
    ];
    for algorithm in ALL_ALGORITHMS {
        let mut validation = Validation::new(algorithm);
        validation.set_required_spec_claims::<&str>(&[]);
        validation.validate_exp = false;
        let res = jsonwebtoken::decode::<Value>(token, key, &validation)
            .map(|x| x.claims)
            .map_err(|e| e.into_kind());
        if !matches!(res, Err(JwtErrorKind::InvalidAlgorithm)) {
            return res;
        }
    }
    unreachable!("JWT Validation Algorithms should be finite");
}

async fn decode(token: String) -> Result<impl IntoResponse, StatusCode> {
    let key = DecodingKey::from_rsa_pem(SANTA_PUBKEY).unwrap();

    match try_algorithms(&token, &key) {
        Ok(value) => Ok(Json(value)),
        Err(JwtErrorKind::InvalidSignature) => Err(StatusCode::UNAUTHORIZED),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

#[inline(always)]
pub fn router() -> Router {
    Router::new()
        .route("/wrap", post(wrap))
        .route("/unwrap", get(unwrap))
        .route("/decode", post(decode))
}
