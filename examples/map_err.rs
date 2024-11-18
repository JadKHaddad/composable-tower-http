//! Run with
//!
//! ```not_rust
//! cargo run --example map_err --features="axum"
//! ```
//!

use std::collections::HashSet;

use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use composable_tower_http::{
    authorize::{
        authorizers::basic_auth::impls::{
            basic_auth_user::BasicAuthUser,
            default_basic_auth_authorizer::DefaultBasicAuthAuthorizer,
        },
        header::basic_auth::DefaultBasicAuthExtractor,
    },
    extension::ExtensionLayerExt,
    extract::ExtractorExt,
};
use http::StatusCode;

#[path = "../util/util.rs"]
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("map_err")?;

    let basic_auth_users: HashSet<BasicAuthUser> = [("user-1", "password-1"), ("user-2", "")]
        .into_iter()
        .map(Into::into)
        .collect();

    let authorizer =
        DefaultBasicAuthAuthorizer::new(DefaultBasicAuthExtractor::new(), basic_auth_users);

    let layer = authorizer.clone().layer();

    let map_err_layer = authorizer.clone().map_err(|_| MyBasicAuthError).layer();

    let app = Router::new()
        // curl -u "user-1:wrong" localhost:5000
        .route("/", get(|| async { "Index" }).layer(layer))
        // curl -u "user-1:wrong" localhost:5000/map_err
        .route("/map_err", get(|| async { "Index" }).layer(map_err_layer))
        .layer(util::trace_layer());

    util::serve(app).await
}

#[derive(Debug, thiserror::Error)]
#[error("Basic auth error")]
struct MyBasicAuthError;

impl IntoResponse for MyBasicAuthError {
    fn into_response(self) -> Response {
        (StatusCode::IM_A_TEAPOT, "Are you a teapot?").into_response()
    }
}

impl From<MyBasicAuthError> for Response {
    fn from(value: MyBasicAuthError) -> Self {
        value.into_response()
    }
}
