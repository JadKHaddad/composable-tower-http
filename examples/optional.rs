//! Run with
//!
//! ```not_rust
//! cargo run --example optional --features="axum"
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
    format!("You used the api key: {:?}", api_key)
}

async fn api_key_optional(Extracted(api_key): Extracted<Option<ApiKey>>) -> impl IntoResponse {
    format!("You used the api key: {:?}", api_key)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("optional")?;

    let valid_api_keys: HashSet<ApiKey> = ["api-key-1", "api-key-2"]
        .into_iter()
        .map(ApiKey::new)
        .collect();

    let authorizer =
        DefaultApiKeyAuthorizer::new(DefaultHeaderExtractor::new("x-api-key"), valid_api_keys);

    let layer = authorizer.clone().layer();

    let optional_layer = authorizer.clone().optional().layer();

    let app = Router::new()
        // curl -H "x-api-key: api-key-1" localhost:5000
        .route("/", get(api_key).layer(layer))
        // curl -H "x-api-key: api-key-1" localhost:5000/optional
        .route("/optional", get(api_key_optional).layer(optional_layer))
        // curl -H "x-api-key: wrong" localhost:5000
        // curl -H "x-api-key: wrong" localhost:5000/optional
        .layer(util::trace_layer());

    util::serve(app).await
}
