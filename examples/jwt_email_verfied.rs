//! Run with
//!
//! ```not_rust
//! cargo run --example jwt_email_verfied --features="axum"
//! ```
//!

use anyhow::Context;
use axum::{response::IntoResponse, routing::get, Json, Router};
use composable_tower_http::{
    authorize::{
        authorizers::jwt::{
            impls::{default_jwt_authorizer::DefaultJwtAuthorizerBuilder, validation::Validation},
            jwk_set::impls::rotating::{
                impls::http_jwk_set_fetcher::HttpJwkSetFetcher,
                rotating_jwk_set_provider::RotatingJwkSetProvider,
            },
        },
        header::bearer::impls::default_bearer_extractor::DefaultBearerExtractor,
    },
    chain::Chain,
    extension::layer::ExtensionLayerExt,
    extract::{extracted::Extracted, extractor::ExtractorExt},
};
use http::StatusCode;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[path = "../util/util.rs"]
mod util;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub email_verified: bool,
    pub name: String,
    pub preferred_username: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
}

async fn claims(Extracted(claims): Extracted<Claims>) -> impl IntoResponse {
    Json(claims)
}

async fn claims_email_verified(Extracted(claims): Extracted<Claims>) -> impl IntoResponse {
    Json(claims)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("jwt_email_verfied")?;

    let jwks_uri = std::env::var("JWKS_URI").unwrap_or_else(|_| {
        String::from("https://keycloak.com/realms/master/protocol/openid-connect/certs")
    });

    let iss =
        std::env::var("ISS").unwrap_or_else(|_| String::from("https://keycloak.com/realms/master"));

    tracing::info!(%jwks_uri, %iss);

    let authorizer = DefaultJwtAuthorizerBuilder::build::<Claims>(
        DefaultBearerExtractor::new(),
        RotatingJwkSetProvider::new(30, HttpJwkSetFetcher::new(jwks_uri, Client::new()))
            .await
            .context("Failed to create jwk set provider")?,
        Validation::new().aud(&["account"]).iss(&[iss]),
    );

    let layer = authorizer.clone().layer();

    let chain_layer = authorizer.clone().chain(EmailVerifier::new()).layer();

    let app = Router::new()
        // curl -H "Authorization: Bearer <token>" localhost:5000
        .route("/", get(claims).layer(layer))
        // curl -H "Authorization: Bearer <token>" localhost:5000/chain
        .route("/chain", get(claims_email_verified).layer(chain_layer))
        .layer(util::trace_layer());

    util::serve(app).await
}

#[derive(Debug)]
struct EmailVerifier;

impl EmailVerifier {
    fn new() -> Self {
        Self
    }
}

impl Chain<Claims> for EmailVerifier {
    type Extracted = Claims;

    type Error = EmailVerificationError;

    async fn chain(&self, value: Claims) -> Result<Self::Extracted, Self::Error> {
        if value.email_verified {
            return Ok(value);
        }

        Err(EmailVerificationError)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Email not verified")]
struct EmailVerificationError;

impl IntoResponse for EmailVerificationError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::UNAUTHORIZED, "Email not verified").into_response()
    }
}