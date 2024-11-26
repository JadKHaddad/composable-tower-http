//! Run with
//!
//! ```not_rust
//! cargo run --example any --features="axum"
//! ```
//!

use std::collections::HashSet;

use axum::{response::IntoResponse, routing::get, Router};
use composable_tower_http::{
    authorize::{
        api_key::{ApiKey, DefaultApiKeyAuthorizer},
        header::DefaultHeaderExtractor,
    },
    extension::ExtensionLayerExt,
    extract::{Extracted, ExtractorExt},
};

#[path = "../util/util.rs"]
mod util;

async fn api_key(Extracted(api_key): Extracted<ApiKey>) -> impl IntoResponse {
    format!("You used the api key: {}", api_key.value)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("any")?;

    let x_valid_api_keys: HashSet<ApiKey> = ["api-key-1-x", "api-key-2-x"]
        .into_iter()
        .map(ApiKey::new)
        .collect();

    let y_valid_api_keys: HashSet<ApiKey> = ["api-key-1-y", "api-key-2-y"]
        .into_iter()
        .map(ApiKey::new)
        .collect();

    let z_valid_api_keys: HashSet<ApiKey> = ["api-key-1-z", "api-key-2-z"]
        .into_iter()
        .map(ApiKey::new)
        .collect();

    let x_authorizer =
        DefaultApiKeyAuthorizer::new(DefaultHeaderExtractor::new("x-api-key"), x_valid_api_keys);

    let y_authorizer =
        DefaultApiKeyAuthorizer::new(DefaultHeaderExtractor::new("y-api-key"), y_valid_api_keys);

    let z_authorizer =
        DefaultApiKeyAuthorizer::new(DefaultHeaderExtractor::new("z-api-key"), z_valid_api_keys);

    // Extractors chained with `any` must all return the same type, which will be returned by the `Any` extractor.
    let layer = x_authorizer
        .any(y_authorizer)
        .any(z_authorizer)
        .extension_layer();

    let app = Router::new()
        // curl -H "x-api-key: api-key-1-x" localhost:5000
        // curl -H "y-api-key: api-key-1-y" localhost:5000
        // curl -H "z-api-key: api-key-1-z" localhost:5000
        .route("/", get(api_key))
        .layer(layer)
        // curl -H "x-api-key: wrong" localhost:5000
        // curl -H "y-api-key: wrong" localhost:5000
        // curl -H "z-api-key: wrong" localhost:5000
        // curl localhost:5000
        .layer(util::trace_layer());

    util::serve(app).await
}
