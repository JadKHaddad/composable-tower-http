#![deny(unsafe_code, missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Highly costumizable http utilities built on top of [tower](https://docs.rs/tower/latest/tower/).

pub mod authorize;
pub mod extension;
pub mod extract;
pub mod validate;
