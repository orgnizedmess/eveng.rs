//! # EVE-NG API client for Rust
//!
//! This library is an API client for [EVE-NG]. It implements the endpoints
//! documented as part of it's [REST API], as well as lesser documented
//! endpoints found in the source code.
//!
//! Tested on the Community Edition Version 6.2.0-4.
//!
//! [EVE-NG]: https://eve-ng.net
//! [REST API]: https://www.eve-ng.net/index.php/how-to-eve-ng-api/

mod client;
mod error;
mod utils;

pub mod folders;
pub mod interfaces;
pub mod labs;
pub mod networks;
pub mod nodes;
pub mod system;
pub mod users;

pub use client::Client;
pub use error::{Error, Result};
