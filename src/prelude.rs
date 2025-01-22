//! # The Quark prelude
//!
//! This prelude contains the bare essentials to get a Quark project up and going. Use it with `use quark::prelude::*;` at the top of your `main.rs` file!

pub use crate::{config::QuarkConfig, error::QuarkError, webview::SizeHint, Quark};
