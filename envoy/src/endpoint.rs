use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::Future;

use crate::middleware::{Next};
use crate::{Middleware};

/// An HTTP request handler.
///
/// This trait is automatically implemented for `Fn` types, and so is rarely implemented
/// directly by Envoy users.
///
/// In practice, endpoints are functions that take a `Request` as an argument and
/// return a type `T` that implements `Into<Response>`.
///
/// Endpoints are implemented as asynchronous functions that make use of language features
/// currently only available in Rust Nightly. For this reason, we have to explicitly enable
/// the attribute will be omitted in most of the documentation.
///
/// A simple endpoint that is invoked on a `GET` request and returns a `String`:
///
/// ```no_run
/// async fn hello(ctx: &mut envoy::Context) -> envoy::Result {
///     Ok(ctx.res.set_body(String::from("hello")))
/// }
///
/// let mut app = envoy::Server::new();
/// app.at("/hello").get(hello);
/// ```
#[async_trait]
pub trait Endpoint: Send + Sync {
    /// Invoke the endpoint within the given context
    async fn call(&self, ctx: &mut crate::Context) -> crate::Result;
}

pub(crate) type DynEndpoint = dyn Endpoint;

impl Debug for DynEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dyn Endpoint<{:?}>", std::any::type_name::<Self>())
    }
}

#[async_trait::async_trait]
impl<F> Endpoint for F
where
    F: for<'a1> Fn1<&'a1 mut crate::Context> + Sync + Send,
    for<'a1> <F as Fn1<&'a1 mut crate::Context>>::Output: Future<Output = crate::Result> + Send,
{
    async fn call(&self, ctx: &mut crate::Context) -> crate::Result {
        self(ctx).await
    }
}

trait Fn1<Arg1>: Fn(Arg1) -> <Self as Fn1<Arg1>>::Output {
    type Output;
}
impl<F: Fn(Arg1) -> O, Arg1, O> Fn1<Arg1> for F {
    type Output = O;
}

pub(crate) struct MiddlewareEndpoint {
    endpoint: Arc<dyn Endpoint>,
    middleware: Arc<Vec<Arc<dyn Middleware>>>,
}

impl Clone for MiddlewareEndpoint {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            middleware: self.middleware.clone(),
        }
    }
}

impl std::fmt::Debug for MiddlewareEndpoint {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            fmt,
            "MiddlewareEndpoint (length: {})",
            self.middleware.len(),
        )
    }
}

impl MiddlewareEndpoint

{
    pub(crate) fn wrap_with_middleware(
        ep: impl Endpoint + 'static,
        middleware: Vec<Arc<dyn Middleware>>,
    ) -> Arc<dyn Endpoint + Send + Sync> {
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
impl Endpoint for MiddlewareEndpoint {
    async fn call(&self, ctx: &mut crate::Context) -> crate::Result {
        let next = Next::new(self.endpoint.clone(), self.middleware.clone());
        next.run(ctx).await
    }
}

#[async_trait]
impl Endpoint for Box<dyn Endpoint> {
    async fn call(&self, ctx: &mut crate::Context) -> crate::Result {
        self.as_ref().call(ctx).await
    }
}
