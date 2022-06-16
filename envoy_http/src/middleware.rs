//! Middleware types.

use std::fmt::Debug;
use std::sync::Arc;

use crate::endpoint::DynEndpoint;
use async_trait::async_trait;
use std::future::Future;

/// Middleware that wraps around the remaining middleware chain.
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Asynchronously handle the request, and return a response.
    async fn handle(&self, ctx: &mut crate::Context, next: Next) -> crate::Result;

    /// Set the middleware's name. By default it uses the type signature.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

impl Debug for dyn Middleware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "dyn Middleware<{:?}>",
            std::any::type_name::<Self>(),
        )
    }
}

#[async_trait::async_trait]
impl<F> Middleware for F
where
    F: for<'arg1> Fn1<&'arg1 mut crate::Context, Next> + Sync + Send,
    for<'arg1> <F as Fn1<&'arg1 mut crate::Context, Next>>::Output: Future<Output = crate::Result> + Send,
{
    async fn handle(&self, ctx: &mut crate::Context, next: Next) -> crate::Result {
        self(ctx, next).await
    }
}

trait Fn1<Arg1, Arg2>: Fn(Arg1, Arg2) -> <Self as Fn1<Arg1, Arg2>>::Output {
    type Output;
}
impl<F: Fn(Arg1, Arg2) -> O, Arg1, Arg2, O> Fn1<Arg1, Arg2> for F {
    type Output = O;
}

/// The remainder of a middleware chain, including the endpoint.
#[derive(Debug)]
pub struct Next {
    endpoint: Arc<DynEndpoint>,
    middleware: Arc<Vec<Arc<dyn Middleware>>>,
    current_index: usize,
}

impl Next {
    /// Create a new Next instance.
    pub fn new(
        endpoint: Arc<DynEndpoint>,
        middleware: Arc<Vec<Arc<dyn Middleware>>>,
    ) -> Next {
        Next {
            endpoint,
            middleware,
            current_index: 0,
        }
    }

    /// Asynchronously execute the remaining middleware chain.
    pub async fn run(mut self, ctx: &mut crate::Context) -> crate::Result {
        let current_index = self.current_index; // get a copy of the current index
        self.current_index += 1; // increment the index for the next call

        match self.middleware.get(current_index) {
            // if there is a next middleware
            Some(current) => current.clone().handle(ctx, self).await,
            // if there is no next middleware, execute the endpoint
            None => self.endpoint.call(ctx).await,
        }
    }
}
