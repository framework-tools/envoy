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
//! #[async_std::main]
//! async fn main() -> envoy::Result<()> {
//!     let mut app = envoy::new();
//!     app.at("/orders/shoes").post(order_shoes);
//!     app.listen("127.0.0.1:8080").await?;
//!     Ok(())
//! }
//!
//! async fn order_shoes(mut ctx: Context<()>) -> envoy::Result {
//!     let Animal { name, legs } = ctx.body_json().await?;
//!     Ok(format!("Hello, {}! I've put in an order for {} shoes", name, legs).into())
//! }
//! ````
//!
//! ```sh
//! $ curl localhost:8080/orders/shoes -d '{ "name": "Chashu", "legs": 4 }'
//! Hello, Chashu! I've put in an order for 4 shoes
//!
//! $ curl localhost:8080/orders/shoes -d '{ "name": "Mary Millipede", "legs": 750 }'
//! Hello, Mary Millipede! I've put in an order for 750 shoes
//! ```
//! See more examples in the [examples](https://github.com/http-rs/envoy/tree/main/examples) directory.

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
mod redirect;
mod request;
mod response;
mod response_builder;
mod route;
mod router;
mod server;
pub mod convert;
pub mod listener;
pub mod prelude;

pub use endpoint::Endpoint;
pub use middleware::{Middleware, Next};
pub use redirect::Redirect;
pub use request::Request;
pub use response::Response;
pub use response_builder::ResponseBuilder;
pub use route::Route;
pub use server::Server;
pub use context::Context;

pub use http_types::{self as http, Body, Error, Status, StatusCode};

/// Create a new Envoy server.
///
/// # Examples
///
/// ```no_run
/// # use async_std::task::block_on;
/// # fn main() -> Result<(), std::io::Error> { block_on(async {
/// #
/// let mut app = envoy::new();
/// app.at("/").get(|_| async { Ok("Hello, world!") });
/// app.listen("127.0.0.1:8080").await?;
/// #
/// # Ok(()) }) }
/// ```
#[must_use]
pub fn new<Err>() -> server::Server<(), Err> {
    Server::new()
}

/// Create a new Envoy server with shared application scoped state.
///
/// Application scoped state is useful for storing items
///
/// # Examples
///
/// ```no_run
/// # use async_std::task::block_on;
/// # fn main() -> Result<(), std::io::Error> { block_on(async {
/// #
/// use envoy::Context;
///
/// /// The shared application state.
/// #[derive(Clone)]
/// struct State {
///     name: String,
/// }
///
/// // Define a new instance of the state.
/// let state = State {
///     name: "Nori".to_string()
/// };
///
/// // Initialize the application with state.
/// let mut app = envoy::with_state(state);
/// app.at("/").get(|ctx: Context<State>| async move {
///     Ok(format!("Hello, {}!", &ctx.state().name))
/// });
/// app.listen("127.0.0.1:8080").await?;
/// #
/// # Ok(()) }) }
/// ```
pub fn with_state<State, Err: EnvoyErr>(state: State) -> server::Server<State, Err>
where State: Clone + Send + Sync + 'static,
{
    Server::with_state(state)
}

/// A specialized Result type for Envoy.
pub type Result<Error> = std::result::Result<(), HttpError<Error>>;
pub trait EnvoyErr: Send + Sync + core::fmt::Debug + 'static {}
impl<T> EnvoyErr for T where T: Send + Sync + core::fmt::Debug + 'static {}

#[derive(Debug)]
struct HttpError<Error> {
    status: StatusCode,
    error: Error,
}
