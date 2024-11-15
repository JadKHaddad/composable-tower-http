//! Run with
//!
//! ```not_rust
//! cargo run --example validate --features="axum"
//! ```
//!

#[path = "../util/util.rs"]
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init("validate")?;

    Ok(())
}
