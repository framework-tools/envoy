//! Miscellaneous utilities.

use crate::{Middleware, Next, Response, Context};
pub use async_trait::async_trait;
use std::future::Future;

/// Define a middleware that operates on incoming requests.
///
/// This middleware is useful because it is not possible in Rust yet to use
/// closures to define inline middleware.
///
/// # Examples
///
/// ```rust
/// use envoy::{utils, Context};
/// use std::time::Instant;
///
/// let mut app = envoy::new();
/// app.with(utils::Before(|mut ctx: Context<()>| async move {
///     ctx.set_ext(Instant::now());
///     ctx
/// }));
/// ```
#[derive(Debug)]
pub struct Before<F>(pub F);

#[async_trait]
impl<State, F, Fut> Middleware<State> for Before<F>
where
    State: Clone + Send + Sync + 'static,
    F: Fn(Context<State>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Context<State>> + Send + Sync + 'static,
{
    async fn handle(&self, ctx: crate::Context<State>, next: Next<State>) -> crate::Result {
        let ctx = (self.0)(ctx).await;
        Ok(next.run(ctx).await)
    }
}

/// Define a middleware that operates on outgoing responses.
///
/// This middleware is useful because it is not possible in Rust yet to use
/// closures to define inline middleware.
///
/// # Examples
///
/// ```rust
/// use envoy::{utils, http, Response};
///
/// let mut app = envoy::new();
/// app.with(utils::After(|res: Response| async move {
///     match res.status() {
///         http::StatusCode::NotFound => Ok("Page not found".into()),
///         http::StatusCode::InternalServerError => Ok("Something went wrong".into()),
///         _ => Ok(res),
///     }
/// }));
/// ```
#[derive(Debug)]
pub struct After<F>(pub F);
#[async_trait]
impl<State, F, Fut> Middleware<State> for After<F>
where
    State: Clone + Send + Sync + 'static,
    F: Fn(Response) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = crate::Result> + Send + Sync + 'static,
{
    async fn handle(&self, ctx: crate::Context<State>, next: Next<State>) -> crate::Result {
        let response = next.run(ctx).await;
        (self.0)(response).await
    }
}
