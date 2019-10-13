//!Smart pointers for Rust
//!
//!## Features
//!
//!- `alloc` Enables usage of `alloc` crate

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod utils;
pub mod unique;
pub use unique::Unique;
