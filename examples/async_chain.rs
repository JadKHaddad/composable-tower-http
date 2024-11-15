//! Run with
//!
//! ```not_rust
//! cargo run --example jwt --features="axum"
//! ```
//!

use core::error;

use anyhow::Context;
use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use composable_tower_http::{
    authorize::{
        authorizer::AuthorizerExt,
        authorizers::jwt::{
            impls::{default_jwt_authorizer::DefaultJwtAuthorizerBuilder, validation::Validation},
            jwk_set::impls::rotating::{
                impls::http_jwk_set_fetcher::HttpJwkSetFetcher,
                rotating_jwk_set_provider::RotatingJwkSetProvider,
            },
        },
        extract::authorized::Authorized,
        header::bearer::impls::default_bearer_extractor::DefaultBearerExtractor,
    },
    extension::layer::ExtensionLayerExt,
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

async fn claims(Authorized(claims): Authorized<Claims>) -> impl IntoResponse {
    Json(claims)
}

// async fn claims_email_verified(Authorized(claims): Authorized<Claims>) -> impl IntoResponse {
//     Json(claims)
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("async_chain")?;

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

    let layer = authorizer.clone().extracted().layer();

    let access_service = AccessService::new(Client::new());

    // let async_chain_layer = authorizer
    //     .clone()
    //     .async_chain(|claims: Claims| async move { access_service.authorize(claims).await })
    //     .extracted()
    //     .layer();

    // let chain_layer = authorizer
    //     .clone()
    //     .chain(|claims: Claims| {
    //         if claims.email_verified {
    //             return Ok(claims);
    //         }

    //         Err(EmailVerifiedError::Verify)
    //     })
    //     .extracted()
    //     .layer();

    let app = Router::new()
        // curl -H "Authorization: Bearer <token>" localhost:5000
        .route("/", get(claims).layer(layer))
        // curl -H "Authorization: Bearer <token>" localhost:5000/chain
        // .route("/chain", get(claims_email_verified).layer(chain_layer))
        .layer(util::trace_layer());

    util::serve(app).await
}

#[derive(Debug, Clone)]
struct AccessService {
    client: Client,
}

impl AccessService {
    fn new(client: Client) -> Self {
        Self { client }
    }

    async fn check_access(&self, _email: &str) -> anyhow::Result<bool> {
        let random_numbers = self
            .client
            .get("http://www.randomnumberapi.com/api/v1.0/random?min=0&max=10&count=1")
            .send()
            .await
            .context("Failed to send request")?
            .json::<Vec<u32>>()
            .await
            .context("Failed to parse response")?;

        let number = random_numbers.first().context("No number in response")?;

        Ok(*number % 2 == 0)
    }

    async fn authorize<A>(&self, claims: Claims) -> Result<Claims, AccessError<A>> {
        if self
            .check_access(&claims.email)
            .await
            .map_err(AccessError::Api)?
        {
            return Ok(claims);
        };

        Err(AccessError::Access)
    }
}

#[derive(Debug, thiserror::Error)]
enum AccessError<A> {
    #[error("Authorization error: {0}")]
    Authorize(
        #[source]
        #[from]
        A,
    ),
    #[error("Access denied")]
    Access,
    #[error("Api error: {0}")]
    Api(#[source] anyhow::Error),
}

impl<A> IntoResponse for AccessError<A>
where
    A: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            AccessError::Authorize(err) => err.into_response(),
            AccessError::Access => (StatusCode::FORBIDDEN, "Access denied").into_response(),
            AccessError::Api(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Api error").into_response(),
        }
    }
}

impl<A> From<AccessError<A>> for Response
where
    A: IntoResponse,
{
    fn from(value: AccessError<A>) -> Self {
        value.into_response()
    }
}
