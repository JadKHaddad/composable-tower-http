use std::net::SocketAddr;

use anyhow::Context;
use tokio::net::TcpListener;
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};

pub fn init(exe: &str) -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            format!("{exe}=trace,composable_tower_http=trace,tower_http=trace"),
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

pub fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
}

pub async fn serve(app: axum::Router<()>) -> anyhow::Result<()> {
    let socket_addr = "127.0.0.1:5000".parse::<SocketAddr>()?;

    tracing::info!(%socket_addr, "Starting server");

    let listener = TcpListener::bind(&socket_addr)
        .await
        .context("Bind failed")?;

    axum::serve(listener, app).await.context("Server failed")?;

    Ok(())
}
