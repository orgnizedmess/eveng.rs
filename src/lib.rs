#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(not(any(feature = "rustls", feature = "native-tls")))]
compile_error!("eveng needs a TLS backend: enable the `rustls` or `native-tls` feature");

mod client;
mod error;
mod utils;

pub mod folders;
pub mod interfaces;
pub mod labs;
pub mod networks;
pub mod nodes;
pub mod system;
pub mod templates;
pub mod users;

pub use client::{Client, ClientBuilder};
pub use error::Error;

/// Result type alias.
pub type Result<T> = std::result::Result<T, Error>;
