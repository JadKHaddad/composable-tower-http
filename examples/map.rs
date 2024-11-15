//! Run with
//!
//! ```not_rust
//! cargo run --example map --features="axum"
//! ```
//!

use std::collections::HashSet;

use axum::{response::IntoResponse, routing::get, Router};
use composable_tower_http::{
    authorize::{
        authorizer::AuthorizerExt,
        authorizers::api_key::impls::{
            api_key::ApiKey, default_api_key_authorizer::DefaultApiKeyAuthorizer,
        },
        extract::authorized::Authorized,
        header::impls::default_header_extractor::DefaultHeaderExtractor,
    },
    extension::layer::ExtensionLayerExt,
};

#[path = "../util/util.rs"]
mod util;

async fn api_key(Authorized(api_key): Authorized<String>) -> impl IntoResponse {
    format!("You used the api key: {}", api_key)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("map")?;

    let valid_api_keys: HashSet<ApiKey> = ["api-key-1", "api-key-2"]
        .into_iter()
        .map(ApiKey::new)
        .collect();

    let layer =
        DefaultApiKeyAuthorizer::new(DefaultHeaderExtractor::new("x-api-key"), valid_api_keys)
            .map(|api_key: ApiKey| format!("[mapped {}]", api_key.value))
            .extracted()
            .layer();

    let app = Router::new()
        // curl -H "x-api-key: api-key-1" localhost:5000
        .route("/", get(api_key))
        .layer(layer)
        // curl -H "x-api-key: wrong" localhost:5000
        .layer(util::trace_layer());

    util::serve(app).await
}
