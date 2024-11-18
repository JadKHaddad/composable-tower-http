//! Run with
//!
//! ```not_rust
//! cargo run --example jwt --features="axum"
//! ```
//!

use anyhow::Context;
use axum::{response::IntoResponse, routing::get, Json, Router};
use composable_tower_http::{
    authorize::{
        header::bearer::DefaultBearerExtractor,
        jwt::{
            jwk_set::{fetch::HttpJwkSetFetcher, rotating::RotatingJwkSetProvider},
            DefaultJwtAuthorizerBuilder, Validation,
        },
    },
    extension::ExtensionLayerExt,
    extract::Extracted,
};
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("jwt")?;

    let jwks_uri = std::env::var("JWKS_URI").unwrap_or_else(|_| {
        String::from("https://keycloak.com/realms/master/protocol/openid-connect/certs")
    });

    let iss =
        std::env::var("ISS").unwrap_or_else(|_| String::from("https://keycloak.com/realms/master"));

    tracing::info!(%jwks_uri, %iss);

    let layer = DefaultJwtAuthorizerBuilder::new(
        DefaultBearerExtractor::new(),
        RotatingJwkSetProvider::new(30, HttpJwkSetFetcher::new(jwks_uri, Client::new()))
            .await
            .context("Failed to create jwk set provider")?,
        Validation::new().aud(&["account"]).iss(&[iss]),
    )
    .build::<Claims>()
    .layer();

    let app = Router::new()
        // curl -H "Authorization: Bearer <token>" localhost:5000
        .route("/", get(claims))
        .layer(layer)
        .layer(util::trace_layer());

    util::serve(app).await
}
