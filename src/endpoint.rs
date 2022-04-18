use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;

use crate::middleware::{Next};
use crate::{Middleware, Context, EnvoyErr};

/// An HTTP request handler.
///
/// This trait is automatically implemented for `Fn` types, and so is rarely implemented
/// directly by Envoy users.
///
/// In practice, endpoints are functions that take a `Request<State>` as an argument and
/// return a type `T` that implements `Into<Response>`.
///
/// # Examples
///
/// Endpoints are implemented as asynchronous functions that make use of language features
/// currently only available in Rust Nightly. For this reason, we have to explicitly enable
/// the attribute will be omitted in most of the documentation.
///
/// A simple endpoint that is invoked on a `GET` request and returns a `String`:
///
/// ```no_run
/// async fn hello(_req: envoy::Context<()>) -> envoy::Result<String> {
///     Ok(String::from("hello"))
/// }
///
/// let mut app = envoy::Server::new();
/// app.at("/hello").get(hello);
/// ```
///
/// An endpoint with similar functionality that does not make use of the `async` keyword would look something like this:
///
/// ```no_run
/// # use core::future::Future;
/// fn hello(_req: envoy::Context<()>) -> impl Future<Output = envoy::Result<String>> {
///     async_std::future::ready(Ok(String::from("hello")))
/// }
///
/// let mut app = envoy::Server::new();
/// app.at("/hello").get(hello);
/// ```
///
/// Envoy routes will also accept endpoints with `Fn` signatures of this form, but using the `async` keyword has better ergonomics.
#[async_trait]
pub trait Endpoint<State: Clone + Send + Sync + 'static, Err: EnvoyErr>: Send + Sync + 'static {
    /// Invoke the endpoint within the given context
    async fn call(&self, ctx: Context<State>) -> crate::Result<Err>;
}

pub(crate) type DynEndpoint<State, Err> = dyn Endpoint<State, Err>;

impl<State, Err> Debug for DynEndpoint<State, Err> where State: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dyn {:?}<{:?}>", std::any::type_name::<Self>(), std::any::type_name::<State>())
    }
}

#[async_trait]
impl<State, F, Fut, Err: EnvoyErr> Endpoint<State, Err> for F
where
    State: Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn(Context<State>) -> Fut,
    Fut: core::future::Future<Output = crate::Result<Err>> + Send + 'static,
{
    async fn call(&self, ctx: Context<State>) -> crate::Result<Err> {
        let fut = (self)(ctx);
        let res = fut.await?;
        Ok(res.into())
    }
}

pub(crate) struct MiddlewareEndpoint<E, State, Err> {
    endpoint: Arc<E>,
    middleware: Arc<Vec<Arc<dyn Middleware<State, Err>>>>,
}

impl<E: Clone, State, Err> Clone for MiddlewareEndpoint<E, State, Err> {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            middleware: self.middleware.clone(),
        }
    }
}

impl<E, State, Err> std::fmt::Debug for MiddlewareEndpoint<E, State, Err> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            fmt,
            "MiddlewareEndpoint (length: {})",
            self.middleware.len(),
        )
    }
}

impl<E, State, Err: EnvoyErr> MiddlewareEndpoint<E, State, Err>
where
    State: Clone + Send + Sync + 'static,
    E: Endpoint<State, Err>,
{
    pub(crate) fn wrap_with_middleware(
        ep: E,
        middleware: Vec<Arc<dyn Middleware<State, Err>>>,
    ) -> Arc<dyn Endpoint<State, Err> + Send + Sync + 'static> {
        if middleware.is_empty() {
            Arc::new(ep)
        } else {
            Arc::new(Self {
                endpoint: Arc::new(ep),
                middleware: Arc::new(middleware),
            })
        }
    }
}

#[async_trait]
impl<E, State, Err: EnvoyErr> Endpoint<State, Err> for MiddlewareEndpoint<E, State, Err>
where
    State: Clone + Send + Sync + 'static,
    E: Endpoint<State, Err>,
{
    async fn call(&self, ctx: Context<State>) -> crate::Result<Err> {
        let next = Next::new(self.endpoint.clone(), self.middleware.clone());
        next.run(ctx).await
    }
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static, Err: EnvoyErr> Endpoint<State, Err> for Box<dyn Endpoint<State, Err>> {
    async fn call(&self, ctx: Context<State>) -> crate::Result<Err> {
        self.as_ref().call(ctx).await
    }
}
