//! Run with
//!
//! ```not_rust
//! cargo run --example or_and --features="axum"
//! ```
//!

use axum::{routing::get, Router};

#[path = "../util/util.rs"]
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("or_and")?;

    // TODO
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(util::trace_layer());

    util::serve(app).await
}
