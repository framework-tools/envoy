//! Middleware types.

use std::fmt::Debug;
use std::sync::Arc;

use crate::endpoint::DynEndpoint;
use crate::{Response, Context};
use async_trait::async_trait;
use std::future::Future;

/// Middleware that wraps around the remaining middleware chain.
#[async_trait]
pub trait Middleware<State>: Send + Sync + 'static {
    /// Asynchronously handle the request, and return a response.
    async fn handle(&self, ctx: Context<State>, next: Next<State>) -> crate::Result;

    /// Set the middleware's name. By default it uses the type signature.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

impl<State> Debug for dyn Middleware<State> where State: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dyn {:?}<{:?}>", std::any::type_name::<Self>(), std::any::type_name::<State>())
    }
}


#[async_trait]
impl<State, F, Fut> Middleware<State> for F
where
    State: Clone + Send + Sync + 'static,
    Fut: Future<Output = crate::Result> + Send,
    F: Send + Sync + 'static + Fn(
        Context<State>,
        Next<State>,
    ) -> Fut,
{
    async fn handle(&self, ctx: Context<State>, next: Next<State>) -> crate::Result {
        (self)(ctx, next).await
    }
}

/// The remainder of a middleware chain, including the endpoint.
#[derive(Debug)]
pub struct Next<State> {
    endpoint: Arc<DynEndpoint<State>>,
    middleware: Arc<Vec<Arc<dyn Middleware<State>>>>,
    current_index: usize,
}

impl<State: Clone + Send + Sync + 'static> Next<State> {

    /// Create a new Next instance.
    pub fn new(
        endpoint: Arc<DynEndpoint<State>>,
        middleware: Arc<Vec<Arc<dyn Middleware<State>>>>,
    ) -> Next<State> {
        Next {
            endpoint,
            middleware,
            current_index: 0,
        }
    }

    /// Asynchronously execute the remaining middleware chain.
    pub async fn run(mut self, ctx: Context<State>) -> Response {
        let current_index = self.current_index; // get a copy of the current index
        self.current_index += 1; // increment the index for the next call

        match self.middleware.get(current_index) {
            // if there is a next middleware
            Some(current) => match current.clone().handle(ctx, self).await {
                Ok(request) => request,
                Err(err) => err.into(),
            }
            // if there is no next middleware, execute the endpoint
            None => match self.endpoint.call(ctx).await {
                Ok(request) => request,
                Err(err) => err.into(),
            }
        }
    }
}
