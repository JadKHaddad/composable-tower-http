//! Run with
//!
//! ```not_rust
//! cargo run --example jwt --features="axum"
//! ```
//!

#[path = "../util/util.rs"]
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("jwt")?;

    Ok(())
}
