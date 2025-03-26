#![deny(unsafe_code, missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Highly customizable http utilities built on top of [tower](https://docs.rs/tower/latest/tower/).

pub mod authorize;
pub mod error;
pub mod extension;
pub mod extract;
pub mod modify;

#[cfg(test)]
mod test;
