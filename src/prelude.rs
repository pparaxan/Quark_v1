//! # The Crowsa prelude
//!
//! This prelude contains the bare essentals to get a Crowsa project up and going. Use it with `use crowsa::prelude::*;` at the top of your `main.rs` file!

pub use crate::{Crowsa, config::CrowsaConfig, error::CrowsaError};
pub use hyaline::SizeHint;
