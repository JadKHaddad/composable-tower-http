//! Run with
//!
//! ```not_rust
//! cargo run --example or --features="axum"
//! ```
//!

use std::collections::HashSet;

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
        },
        header::{
            basic_auth::impls::default_basic_auth_extractor::DefaultBaiscAuthExtractor,
            impls::default_header_extractor::DefaultHeaderExtractor,
        },
    },
    extension::layer::ExtensionLayerExt,
    extract::{extracted::Extracted, extractor::ExtractorExt, or::Or},
};

#[path = "../util/util.rs"]
mod util;

async fn api_key_or_basic_auth(
    Extracted(or): Extracted<Or<ApiKey, BasicAuthUser>>,
) -> impl IntoResponse {
    match or {
        Or::Left(api_key) => format!("You used the api key: {:?}", api_key),
        Or::Right(user) => format!("You are: {:?}", user),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("or")?;

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
        DefaultBasicAuthAuthorizer::new(DefaultBaiscAuthExtractor::new(), basic_auth_users);

    let layer = api_key_authorizer.or(basic_auth_authorizer).layer();

    let app = Router::new()
        // curl -H "x-api-key: api-key-1" localhost:5000
        // curl -u "user-1:password-1" localhost:5000
        .route("/", get(api_key_or_basic_auth))
        .layer(layer)
        // curl -H "x-api-key: wrong" localhost:5000
        // curl -u "user-1:wrong" localhost:5000
        // curl localhost:5000
        .layer(util::trace_layer());

    util::serve(app).await
}