//! Run with
//!
//! ```not_rust
//! cargo run --example jwt_random_verification --features="axum"
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

async fn claims_random(Extracted(claims): Extracted<Claims>) -> impl IntoResponse {
    Json(claims)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("jwt_random_verification")?;

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

    let chain_layer = authorizer
        .clone()
        .chain(RandomVerifier::new(Client::new()))
        .layer();

    let app = Router::new()
        // curl -H "Authorization: Bearer <token>" localhost:5000
        .route("/", get(claims).layer(layer))
        // curl -H "Authorization: Bearer <token>" localhost:5000/chain
        .route("/chain", get(claims_random).layer(chain_layer))
        .layer(util::trace_layer());

    util::serve(app).await
}

#[derive(Debug)]
struct RandomVerifier {
    client: Client,
}

impl RandomVerifier {
    fn new(client: Client) -> Self {
        Self { client }
    }
}

impl Chain<Claims> for RandomVerifier {
    type Extracted = Claims;

    type Error = RandomVerificationError;

    async fn chain(&self, value: Claims) -> Result<Self::Extracted, Self::Error> {
        let random_numbers = self
            .client
            .get("http://www.randomnumberapi.com/api/v1.0/random?min=0&max=10&count=1")
            .send()
            .await
            .context("Failed to fetch random number")?
            .json::<Vec<u32>>()
            .await
            .context("Failed to parse response")?;

        let number = random_numbers.first().context("No number in response")?;

        if *number % 2 == 0 {
            return Ok(value);
        }

        Err(RandomVerificationError::Random)
    }
}

#[derive(Debug, thiserror::Error)]

enum RandomVerificationError {
    #[error("Api error: {0}")]
    Api(
        #[source]
        #[from]
        anyhow::Error,
    ),
    #[error("Your number is odd")]
    Random,
}

impl IntoResponse for RandomVerificationError {
    fn into_response(self) -> axum::response::Response {
        match self {
            RandomVerificationError::Api(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
            RandomVerificationError::Random => {
                (StatusCode::UNAUTHORIZED, "Your number is odd").into_response()
            }
        }
    }
}
