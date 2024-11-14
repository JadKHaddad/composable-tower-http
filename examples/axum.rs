//! Run with
//!
//! ```not_rust
//! cargo run --example axum --features="axum"
//! ```
//!

use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use anyhow::Context;
use axum::{response::IntoResponse, routing::get, Json, Router};
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
        extract::{
            authorization_extractor::AuthorizationExtractor, authorized::Authorized,
            sealed_authorized::SealedAuthorized, validated_authorized::ValidatedAuthorized,
        },
        header::{
            basic_auth::impls::default_basic_auth_extractor::DefaultBaiscAuthExtractor,
            bearer::impls::default_bearer_extractor::DefaultBearerExtractor,
            impls::default_header_extractor::DefaultHeaderExtractor,
        },
    },
    extension::layer::{ExtensionLayer, ExtensionLayerExt},
    map::mapper::MapperExt,
    validate::{extract::validation_extractor::ValidationExtractor, validator::Validator},
};
use http::StatusCode;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};

fn init() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "axum=trace,composable_tower_http=trace,tower_http=trace",
        );
    }

    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::Subscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .context("Failed to set global tracing subscriber")?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub email_verified: bool,
    pub name: String,
    pub preferred_username: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
}

async fn claims(Authorized(claims): Authorized<Claims>) -> impl IntoResponse {
    Json(claims)
}

async fn email_verified_claims(
    ValidatedAuthorized(claims): ValidatedAuthorized<Claims>,
) -> impl IntoResponse {
    Json(claims)
}

async fn api_key(Authorized(api_key): Authorized<ApiKey>) -> impl IntoResponse {
    format!("You used the api key: {}", api_key.value)
}

async fn basic_auth(Authorized(user): Authorized<BasicAuthUser>) -> impl IntoResponse {
    format!("You are: {}", user.username)
}

async fn mapped_basic_auth(Authorized(mapped_user): Authorized<String>) -> impl IntoResponse {
    format!("You are: {}", mapped_user)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init()?;

    let jwks_uri = std::env::var("JWKS_URI").unwrap_or_else(|_| {
        String::from("https://keycloak.com/realms/master/protocol/openid-connect/certs")
    });

    let iss =
        std::env::var("ISS").unwrap_or_else(|_| String::from("https://keycloak.com/realms/master"));

    tracing::info!(%jwks_uri, %iss);

    let jwt_authorization_extractor =
        AuthorizationExtractor::new(DefaultJwtAuthorizerBuilder::build::<Claims>(
            DefaultBearerExtractor::new(),
            RotatingJwkSetProvider::new(30, HttpJwkSetFetcher::new(jwks_uri, Client::new()))
                .await
                .context("Failed to create jwk set provider")?,
            Validation::new().aud(&["account"]).iss(&[iss]),
        ));

    let jwt_authorization_layer = ExtensionLayer::new(jwt_authorization_extractor.clone());

    let jwt_validation_authorization_email_verified_layer = {
        #[derive(Clone, Debug)]
        struct EmailVerifiedValidator;

        #[derive(Debug, thiserror::Error)]
        #[error("Email not verified")]
        struct EmailVerifiedError;

        impl IntoResponse for EmailVerifiedError {
            fn into_response(self) -> axum::response::Response {
                (StatusCode::UNAUTHORIZED, "Email not verified").into_response()
            }
        }

        impl Validator<SealedAuthorized<Claims>> for EmailVerifiedValidator {
            type Error = EmailVerifiedError;

            async fn validate(&self, value: &SealedAuthorized<Claims>) -> Result<(), Self::Error> {
                if value.email_verified {
                    return Ok(());
                }

                Err(EmailVerifiedError)
            }
        }

        ExtensionLayer::new(ValidationExtractor::new(
            jwt_authorization_extractor.clone(),
            EmailVerifiedValidator {},
        ))
    };

    let valid_api_keys: HashSet<ApiKey> = ["api-key-1", "api-key-2"]
        .into_iter()
        .map(ApiKey::new)
        .collect();

    let api_key_authorization_layer = AuthorizationExtractor::new(DefaultApiKeyAuthorizer::new(
        DefaultHeaderExtractor::new("x-api-key"),
        valid_api_keys,
    ))
    .extension_layer();

    let basic_auth_users: HashSet<BasicAuthUser> = [("user-1", "password-1"), ("user-2", "")]
        .into_iter()
        .map(Into::into)
        .collect();

    let basic_auth_extractor = AuthorizationExtractor::new(DefaultBasicAuthAuthorizer::new(
        DefaultBaiscAuthExtractor::new(),
        basic_auth_users,
    ));

    let basic_auth_authorization_layer = basic_auth_extractor.clone().extension_layer();

    let mapped_basic_auth_authorization_layer = basic_auth_extractor
        .clone()
        .map(|ex: SealedAuthorized<BasicAuthUser>| ex.map(|_| String::from("A user")))
        .extension_layer();

    let error_mapped_basic_auth_authorization_layer = {
        #[derive(Debug, Clone, thiserror::Error)]
        #[error("Can't let you in")]
        struct BasicAuthError;

        impl IntoResponse for BasicAuthError {
            fn into_response(self) -> axum::response::Response {
                (StatusCode::IM_A_TEAPOT, "Can't let you in").into_response()
            }
        }

        impl From<BasicAuthError> for axum::response::Response {
            fn from(value: BasicAuthError) -> Self {
                value.into_response()
            }
        }

        ExtensionLayer::new(basic_auth_extractor.map_err(|_| BasicAuthError))
    };

    let jwt_app = Router::new()
        .route("/", get(|| async move { "jwt index" }))
        .route("/show_claims", get(claims))
        .layer(jwt_authorization_layer)
        .route("/show_claims_without_layer", get(claims));

    let jwt_email_verified_app = Router::new()
        .route("/", get(|| async move { "jwt index" }))
        .route("/show_claims", get(email_verified_claims))
        .layer(jwt_validation_authorization_email_verified_layer)
        .route("/show_claims_without_layer", get(email_verified_claims));

    let api_key_app = Router::new()
        .route("/", get(|| async move { "api key index" }))
        .route("/show_api_key", get(api_key))
        .layer(api_key_authorization_layer)
        .route("/show_api_key_without_layer", get(api_key));

    // curl -u "user-1:password-1" http://127.0.0.1:5000/basic_auth/show_basic_auth
    // curl -u "user-2" http://127.0.0.1:5000/basic_auth
    let basic_auth_app = Router::new()
        .route("/", get(|| async move { "basic auth index" }))
        .route("/show_basic_auth", get(basic_auth))
        .layer(basic_auth_authorization_layer)
        .route("/show_basic_auth_without_layer", get(basic_auth));

    // curl -u "user-1:password-1" http://127.0.0.1:5000/mapped_basic_auth/show_basic_auth
    let mapped_basic_auth_app = Router::new()
        .route("/", get(|| async move { "mapped basic auth index" }))
        .route("/show_basic_auth", get(mapped_basic_auth))
        .layer(mapped_basic_auth_authorization_layer)
        .route("/show_basic_auth_without_layer", get(mapped_basic_auth));

    // curl -u "user-1:wrong-pass" http://127.0.0.1:5000/error_mapped_basic_auth/show_basic_auth
    let error_mapped_basic_auth_app = Router::new()
        .route("/", get(|| async move { "error mapped basic auth index" }))
        .route("/show_basic_auth", get(basic_auth))
        .layer(error_mapped_basic_auth_authorization_layer)
        .route("/show_basic_auth_without_layer", get(basic_auth));

    let app: Router<()> = Router::new()
        .nest("/jwt", jwt_app)
        .nest("/jwt_email_verified", jwt_email_verified_app)
        .nest("/api_key", api_key_app)
        .nest("/basic_auth", basic_auth_app)
        .nest("/mapped_basic_auth", mapped_basic_auth_app)
        .nest("/error_mapped_basic_auth", error_mapped_basic_auth_app)
        .route("/", get(|| async move { "index" }))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        );

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);

    tracing::info!(%socket_addr, "Starting server");

    let listener = TcpListener::bind(&socket_addr)
        .await
        .context("Bind failed")?;

    axum::serve(listener, app).await.context("Server failed")?;

    Ok(())
}
