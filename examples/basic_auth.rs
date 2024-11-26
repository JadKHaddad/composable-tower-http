//! Run with
//!
//! ```not_rust
//! cargo run --example basic_auth --features="axum"
//! ```
//!

use std::collections::HashSet;

use axum::{response::IntoResponse, routing::get, Router};
use composable_tower_http::{
    authorize::{
        basic_auth::{BasicAuthUser, DefaultBasicAuthAuthorizer},
        header::basic_auth::DefaultBasicAuthExtractor,
    },
    extension::ExtensionLayerExt,
    extract::Extracted,
};

#[path = "../util/util.rs"]
mod util;

async fn basic_auth(Extracted(user): Extracted<BasicAuthUser>) -> impl IntoResponse {
    format!("You are: {:?}", user)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("basic_auth")?;

    let basic_auth_users: HashSet<BasicAuthUser> = [("user-1", "password-1"), ("user-2", "")]
        .into_iter()
        .map(Into::into)
        .collect();

    let layer = DefaultBasicAuthAuthorizer::new(DefaultBasicAuthExtractor::new(), basic_auth_users)
        .extension_layer();

    let app = Router::new()
        // curl -u "user-1:password-1" localhost:5000
        .route("/", get(basic_auth))
        .layer(layer)
        // curl -u "user-1:wrong" localhost:5000
        .layer(util::trace_layer());

    util::serve(app).await
}
