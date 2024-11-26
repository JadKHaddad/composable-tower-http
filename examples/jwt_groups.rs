//! Run with
//!
//! ```not_rust
//! cargo run --example jwt_groups --features="axum"
//! ```
//!

use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use axum::{response::IntoResponse, routing::get, Json, Router};
use composable_tower_http::{
    authorize::{
        header::bearer::DefaultBearerExtractor,
        jwt::{
            jwk_set::{fetch::HttpJwkSetFetcher, rotating::RotatingJwkSetProvider},
            DefaultJwtAuthorizerBuilder, Validation,
        },
    },
    extension::{ExtensionLayerExt, ModificationLayerExt},
    extract::Extracted,
    modify::Modifier,
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
    pub groups: Vec<String>,
}

async fn claims(Extracted(claims): Extracted<Claims>) -> impl IntoResponse {
    Json(claims)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("jwt_groups")?;

    let jwks_uri = std::env::var("JWKS_URI").unwrap_or_else(|_| {
        String::from("https://keycloak.com/realms/master/protocol/openid-connect/certs")
    });

    let iss =
        std::env::var("ISS").unwrap_or_else(|_| String::from("https://keycloak.com/realms/master"));

    tracing::info!(%jwks_uri, %iss);

    let auth_layer = DefaultJwtAuthorizerBuilder::new(
        DefaultBearerExtractor::new(),
        RotatingJwkSetProvider::new(30, HttpJwkSetFetcher::new(jwks_uri, Client::new()))
            .await
            .context("Failed to create jwk set provider")?,
        Validation::new().aud(&["account"]).iss(&[iss]),
    )
    .build::<Claims>()
    .extension_layer();

    // These layers will look into the extracted claims in the request extensions and perform a modification removing the old claims and inserting modified claims.

    let admins: HashSet<String> = ["/admins"].into_iter().map(Into::into).collect();
    let admins_modify_layer = GroupsValidator::new(admins).modification_layer::<Claims>();

    let super_admins: HashSet<String> = ["/super-admins"].into_iter().map(Into::into).collect();
    let super_admins_modify_layer =
        GroupsValidator::new(super_admins).modification_layer::<Claims>();

    let app = Router::new()
        // curl -H "Authorization: Bearer <token>" localhost:5000/super-admins
        .route(
            "/super-admins",
            get(claims).layer(super_admins_modify_layer),
        )
        // curl -H "Authorization: Bearer <token>" localhost:5000/admins
        .route("/admins", get(claims).layer(admins_modify_layer))
        // curl -H "Authorization: Bearer <token>" localhost:5000
        .route("/", get(claims))
        // The auth layer will extract the claims from the request and insert them into the request extensions.
        .layer(auth_layer)
        .layer(util::trace_layer());

    util::serve(app).await
}

#[derive(Debug, Clone)]
struct GroupsValidator {
    groups: Arc<HashSet<String>>,
}

impl GroupsValidator {
    fn new(groups: HashSet<String>) -> Self {
        Self {
            groups: Arc::new(groups),
        }
    }
}

impl Modifier<Claims> for GroupsValidator {
    type Modified = Claims;

    type Error = GroupsValidationError;

    async fn modify(&self, claims: Claims) -> Result<Claims, Self::Error> {
        if claims
            .groups
            .iter()
            .any(|group| self.groups.contains(group))
        {
            return Ok(claims);
        };

        Err(GroupsValidationError)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Not in groups")]
struct GroupsValidationError;

impl IntoResponse for GroupsValidationError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::UNAUTHORIZED, "Not in groups").into_response()
    }
}
