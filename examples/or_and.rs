//! Run with
//!
//! ```not_rust
//! cargo run --example or_and --features="axum"
//! ```
//! This example demonstrates how to use the `Or` and `And` extractors to combine multiple authorizers.
//!
//! The endpoint `/` can be accessed with either (a valid JWT) or (an API key and basic auth).
//!

use std::collections::HashSet;

use anyhow::Context;
use axum::{response::IntoResponse, routing::get, Router};
use composable_tower_http::{
    authorize::{
        authorizers::{
            api_key::impls::{
                api_key::ApiKey, default_api_key_authorizer::DefaultApiKeyAuthorizer,
            },
            basic_auth::impls::{
                basic_auth_user::BasicAuthUser,
                default_basic_auth_authorizer::DefaultBasicAuthAuthorizer,
            },
            jwt::{
                impls::{
                    default_jwt_authorizer::DefaultJwtAuthorizerBuilder, validation::Validation,
                },
                jwk_set::impls::rotating::{
                    impls::http_jwk_set_fetcher::HttpJwkSetFetcher,
                    rotating_jwk_set_provider::RotatingJwkSetProvider,
                },
            },
        },
        header::{
            basic_auth::DefaultBasicAuthExtractor, bearer::DefaultBearerExtractor,
            DefaultHeaderExtractor,
        },
    },
    extension::ExtensionLayerExt,
    extract::{And, Extracted, Extractor, ExtractorExt, Or},
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

async fn index(
    Extracted(or): Extracted<Or<Claims, And<ApiKey, BasicAuthUser>>>,
) -> impl IntoResponse {
    match or {
        Or::Left(claims) => format!("You used a JWT, claims: {:?}", claims),
        Or::Right(And { left, right }) => {
            format!("You used the api key: {:?}, and you are: {:?}", left, right)
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("or_and")?;

    let valid_api_keys: HashSet<ApiKey> = ["api-key-1", "api-key-2"]
        .into_iter()
        .map(ApiKey::new)
        .collect();

    let api_key_authorizer =
        DefaultApiKeyAuthorizer::new(DefaultHeaderExtractor::new("x-api-key"), valid_api_keys);

    let basic_auth_users: HashSet<BasicAuthUser> = [("user-1", "password-1"), ("user-2", "")]
        .into_iter()
        .map(Into::into)
        .collect();

    let basic_auth_authorizer =
        DefaultBasicAuthAuthorizer::new(DefaultBasicAuthExtractor::new(), basic_auth_users);

    let jwks_uri = std::env::var("JWKS_URI").unwrap_or_else(|_| {
        String::from("https://keycloak.com/realms/master/protocol/openid-connect/certs")
    });

    let iss =
        std::env::var("ISS").unwrap_or_else(|_| String::from("https://keycloak.com/realms/master"));

    tracing::info!(%jwks_uri, %iss);

    let jwt_authorizer = DefaultJwtAuthorizerBuilder::new(
        DefaultBearerExtractor::new(),
        RotatingJwkSetProvider::new(30, HttpJwkSetFetcher::new(jwks_uri, Client::new()))
            .await
            .context("Failed to create jwk set provider")?,
        Validation::new().aud(&["account"]).iss(&[iss]),
    )
    .build::<Claims>();

    let autorizer = jwt_authorizer.or(api_key_authorizer.and(basic_auth_authorizer));

    // If things got too complicated, you can always check the extracted type.
    tracing::debug!(
        "The extracted type name is: {}",
        autorizer.extracted_type_name()
    );

    let layer = autorizer.layer();

    let app = Router::new()
        // curl -u "user-1:password-1" -H "x-api-key: api-key-1" localhost:5000
        // curl -H "Authorization: Bearer <token>" localhost:5000
        .route("/", get(index).layer(layer))
        // curl localhost:5000
        .layer(util::trace_layer());

    util::serve(app).await
}
