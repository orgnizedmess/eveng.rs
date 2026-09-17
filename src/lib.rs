#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

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

pub use client::Client;
pub use error::Error;

/// Result type alias.
pub type Result<T> = std::result::Result<T, Error>;
