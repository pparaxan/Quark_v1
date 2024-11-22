//! # The Quark prelude
//!
//! This prelude contains the bare essentals to get a Quark project up and going. Use it with `use Quark::prelude::*;` at the top of your `main.rs` file!

pub use crate::{Quark, config::QuarkConfig, error::QuarkError};
pub use hyaline::SizeHint;
