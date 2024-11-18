//! Run with
//!
//! ```not_rust
//! cargo run --example and --features="axum"
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
    extension::ExtensionLayerExt,
    extract::{And, Extracted, ExtractorExt},
};

#[path = "../util/util.rs"]
mod util;

async fn api_key_and_basic_auth(
    Extracted(And { left, right }): Extracted<And<ApiKey, BasicAuthUser>>,
) -> impl IntoResponse {
    format!("You used the api key: {:?}, and you are: {:?}", left, right)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("and")?;

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

    // This is very similar to chaining layers,
    // but the `And` extractor will contain both extracted values and will prevent similar extracted types from overlapping.
    // You can chain `And` and `Or` extractors to create complex authorization logic. See `or_and` example.
    let layer = api_key_authorizer.and(basic_auth_authorizer).layer();

    let app = Router::new()
        // curl -u "user-1:password-1" -H "x-api-key: api-key-1" localhost:5000
        .route("/", get(api_key_and_basic_auth))
        .layer(layer)
        // curl -u "user-1:wrong" -H "x-api-key: api-key-1" localhost:5000
        // curl -u "user-1:password-1" -H "x-api-key: wrong" localhost:5000
        // curl localhost:5000
        .layer(util::trace_layer());

    util::serve(app).await
}
