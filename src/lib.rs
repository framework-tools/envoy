//! Envoy is a minimal and pragmatic Rust web application framework built for
//! rapid development. It comes with a robust set of features that make
//! building async web applications and APIs easier and more fun.
//!
//! # Getting started
//!
//! In order to build a web app in Rust you need an HTTP server, and an async
//! runtime. After running `cargo init` add the following lines to your
//! `Cargo.toml` file:
//!
//! ```toml
//! # Example, use the version numbers you need
//! envoy = "0.14.0"
//! async-std = { version = "1.6.0", features = ["attributes"] }
//! serde = { version = "1.0", features = ["derive"] }
//!```
//!
//! # Examples
//!
//! Create an HTTP server that receives a JSON body, validates it, and responds with a
//! confirmation message.
//!
//! ```no_run
//! use envoy::Context;
//! use envoy::prelude::*;
//!
//! #[derive(Debug, Deserialize)]
//! struct Animal {
//!     name: String,
//!     legs: u16,
//! }
//!
//! #[tokio::main]
//! async fn main() -> envoy::Result {
//!     let mut app = envoy::new();
//!     app.at("/orders/shoes").post(order_shoes);
//!     app.listen("127.0.0.1:8080").await?;
//!     Ok(())
//! }
//!
//! async fn order_shoes(ctx: &mut Context) -> envoy::Result {
//!     let Animal { name, legs } = ctx.body_json().await?;
//!     Ok(ctx.res.set_body(format!("Hello, {}! I've put in an order for {} shoes", name, legs)))
//! }
//! ````

#![cfg_attr(feature = "docs", feature(doc_cfg))]
#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, unreachable_pub, future_incompatible, rust_2018_idioms)]
#![allow(clippy::len_without_is_empty)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]
#![doc(html_favicon_url = "https://yoshuawuyts.com/assets/http-rs/favicon.ico")]
#![doc(html_logo_url = "https://yoshuawuyts.com/assets/http-rs/logo-rounded.png")]


mod context;
mod endpoint;
mod middleware;
mod route;
mod router;
mod server;
pub mod convert;
pub mod listener;
pub mod prelude;

pub use endpoint::Endpoint;
pub use middleware::{Middleware, Next};
pub use route::Route;
pub use server::Server;
pub use context::Context;

pub use http_types::{self as http, Body, Error, Status, StatusCode};

/// Create a new Envoy server.
#[must_use]
pub fn new() -> server::Server {
    Server::new()
}

/// A specialized Result type for Envoy.
pub type Result<T = ()> = std::result::Result<T, crate::http::Error>;