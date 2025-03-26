//! Run with
//!
//! ```not_rust
//! cargo run --example jwt_email_verified_lite --features="axum"
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
    extract::{Extracted, ExtractorExt},
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
    util::init("jwt_email_verified_lite")?;

    let jwks_uri = std::env::var("JWKS_URI").unwrap_or_else(|_| {
        String::from("https://keycloak.com/realms/master/protocol/openid-connect/certs")
    });

    let iss =
        std::env::var("ISS").unwrap_or_else(|_| String::from("https://keycloak.com/realms/master"));

    tracing::info!(%jwks_uri, %iss);

    let authorizer = DefaultJwtAuthorizerBuilder::new(
        DefaultBearerExtractor::new(),
        RotatingJwkSetProvider::new(30, HttpJwkSetFetcher::new(jwks_uri, Client::new()))
            .await
            .context("Failed to create jwk set provider")?,
        Validation::new().aud(&["account"]).iss(&[iss]),
    )
    .build::<Claims>();

    let layer = authorizer.clone().extension_layer();

    let chain_layer = authorizer
        .clone()
        .chain_lite(|claims: Claims| {
            if claims.email_verified {
                return Ok(claims);
            }

            Err(EmailVerificationError::Verify)
        })
        .extension_layer();

    let app = Router::new()
        // curl -H "Authorization: Bearer <token>" localhost:5000
        .route("/", get(claims).layer(layer))
        // curl -H "Authorization: Bearer <token>" localhost:5000/chain
        .route("/chain", get(claims_email_verified).layer(chain_layer))
        .layer(util::trace_layer());

    util::serve(app).await
}

#[derive(Debug, thiserror::Error)]
enum EmailVerificationError<A> {
    #[error("Extraction error: {0}")]
    Extract(
        #[source]
        #[from]
        A,
    ),
    #[error("Email not verified")]
    Verify,
}

impl<A> IntoResponse for EmailVerificationError<A>
where
    A: IntoResponse,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            EmailVerificationError::Extract(err) => err.into_response(),
            EmailVerificationError::Verify => {
                (StatusCode::FORBIDDEN, "Email not verified").into_response()
            }
        }
    }
}

impl<A> From<EmailVerificationError<A>> for axum::response::Response
where
    A: IntoResponse,
{
    fn from(value: EmailVerificationError<A>) -> Self {
        value.into_response()
    }
}
