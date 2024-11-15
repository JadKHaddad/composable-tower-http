//! Run with
//!
//! ```not_rust
//! cargo run --example basic_auth --features="axum"
//! ```
//!

use axum::{routing::get, Router};

#[path = "../util/util.rs"]
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("basic_auth")?;

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(util::trace_layer());

    util::serve(app).await
}
