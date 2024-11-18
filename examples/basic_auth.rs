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
        authorizers::basic_auth::impls::{
            basic_auth_user::BasicAuthUser,
            default_basic_auth_authorizer::DefaultBasicAuthAuthorizer,
        },
        header::basic_auth::impls::default_basic_auth_extractor::DefaultBaiscAuthExtractor,
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

    let layer =
        DefaultBasicAuthAuthorizer::new(DefaultBaiscAuthExtractor::new(), basic_auth_users).layer();

    let app = Router::new()
        // curl -u "user-1:password-1" localhost:5000
        .route("/", get(basic_auth))
        .layer(layer)
        // curl -u "user-1:wrong" localhost:5000
        .layer(util::trace_layer());

    util::serve(app).await
}
