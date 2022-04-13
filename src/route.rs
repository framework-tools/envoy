use std::fmt::Debug;
use std::sync::Arc;

use crate::endpoint::MiddlewareEndpoint;
use crate::log;
use crate::{router::Router, Endpoint, Middleware};

/// A handle to a route.
///
/// All HTTP requests are made against resources. After using [`Server::at`] (or
/// [`Route::at`]) to establish a route, the `Route` type can be used to
/// establish endpoints for various HTTP methods at that path. Also, using
/// `nest`, it can be used to set up a subrouter.
///
/// [`Server::at`]: ./struct.Server.html#method.at
#[allow(missing_debug_implementations)]
pub struct Route<'a, State> {
    router: &'a mut Router<State>,
    path: String,
    middleware: Vec<Arc<dyn Middleware<State>>>,
    /// Indicates whether the path of current route is treated as a prefix. Set by
    /// [`strip_prefix`].
    ///
    /// [`strip_prefix`]: #method.strip_prefix
    prefix: bool,
}

impl<'a, State: Clone + Send + Sync + 'static> Route<'a, State> {
    pub(crate) fn new(router: &'a mut Router<State>, path: String) -> Route<'a, State> {
        Route {
            router,
            path,
            middleware: Vec::new(),
            prefix: false,
        }
    }

    /// Extend the route with the given `path`.
    pub fn at<'b>(&'b mut self, path: &str) -> Route<'b, State> {
        let mut p = self.path.clone();

        if !p.ends_with('/') && !path.starts_with('/') {
            p.push('/');
        }

        if path != "/" {
            p.push_str(path);
        }

        Route {
            router: self.router,
            path: p,
            middleware: self.middleware.clone(),
            prefix: false,
        }
    }

    /// Get the current path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Treat the current path as a prefix, and strip prefixes from requests.
    ///
    /// This method is marked unstable as its name might change in the near future.
    ///
    /// Endpoints will be given a path with the prefix removed.
    #[cfg(any(feature = "unstable", feature = "docs"))]
    #[cfg_attr(feature = "docs", doc(cfg(unstable)))]
    pub fn strip_prefix(&mut self) -> &mut Self {
        self.prefix = true;
        self
    }

    /// Apply the given middleware to the current route.
    pub fn with<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State>,
    {
        log::trace!(
            "Adding middleware {} to route {:?}",
            middleware.name(),
            self.path
        );
        self.middleware.push(Arc::new(middleware));
        self
    }

    /// Reset the middleware chain for the current route, if any.
    pub fn reset_middleware(&mut self) -> &mut Self {
        self.middleware.clear();
        self
    }

    /// Nest a [`Server`] at the current path.
    ///
    /// # Note
    ///
    /// The outer server *always* has precedence when disambiguating
    /// overlapping paths. For example in the following example `/hello` will
    /// return "Unexpected" to the client
    ///
    /// ```no_run
    /// #[async_std::main]
    /// async fn main() -> Result<(), std::io::Error> {
    ///     let mut app = envoy::new();
    ///     app.at("/hello").nest({
    ///         let mut example = envoy::with_state("world");
    ///         example
    ///             .at("/")
    ///             .get(|req: envoy::Context<&'static str>| async move {
    ///                 Ok(format!("Hello {state}!", state = req.state()))
    ///             });
    ///         example
    ///     });
    ///     app.at("/*").get(|_| async { Ok("Unexpected") });
    ///     app.listen("127.0.0.1:8080").await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// [`Server`]: struct.Server.html
    pub fn nest<InnerState>(&mut self, service: crate::Server<InnerState>) -> &mut Self
    where
        State: Clone + Send + Sync + 'static,
        InnerState: Clone + Send + Sync + 'static,
    {
        let prefix = self.prefix;

        self.prefix = true;
        self.all(service);
        self.prefix = prefix;

        self
    }

    /// Add an endpoint for the given HTTP method
    pub fn method(&mut self, method: http_types::Method, ep: impl Endpoint<State>) -> &mut Self {
        if self.prefix {
            let ep = StripPrefixEndpoint::new(ep);
            let wildcard = self.at("*");
            wildcard.router.add(
                &wildcard.path,
                method,
                MiddlewareEndpoint::wrap_with_middleware(ep, wildcard.middleware),
            );
        } else {
            self.router.add(
                &self.path,
                method,
                MiddlewareEndpoint::wrap_with_middleware(ep, self.middleware.clone()),
            );
        }
        self
    }

    /// Add an endpoint for all HTTP methods, as a fallback.
    ///
    /// Routes with specific HTTP methods will be tried first.
    pub fn all(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        if self.prefix {
            let ep = StripPrefixEndpoint::new(ep);
            let wildcard = self.at("*");
            wildcard.router.add_all(
                &wildcard.path,
                MiddlewareEndpoint::wrap_with_middleware(ep, wildcard.middleware),
            );
        } else {
            self.router.add_all(
                &self.path,
                MiddlewareEndpoint::wrap_with_middleware(ep, self.middleware.clone()),
            );
        }
        self
    }

    /// Add an endpoint for `GET` requests
    pub fn get(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Get, ep);
        self
    }

    /// Add an endpoint for `HEAD` requests
    pub fn head(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Head, ep);
        self
    }

    /// Add an endpoint for `PUT` requests
    pub fn put(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Put, ep);
        self
    }

    /// Add an endpoint for `POST` requests
    pub fn post(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Post, ep);
        self
    }

    /// Add an endpoint for `DELETE` requests
    pub fn delete(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Delete, ep);
        self
    }

    /// Add an endpoint for `OPTIONS` requests
    pub fn options(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Options, ep);
        self
    }

    /// Add an endpoint for `CONNECT` requests
    pub fn connect(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Connect, ep);
        self
    }

    /// Add an endpoint for `PATCH` requests
    pub fn patch(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Patch, ep);
        self
    }

    /// Add an endpoint for `TRACE` requests
    pub fn trace(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Trace, ep);
        self
    }
}

#[derive(Debug)]
struct StripPrefixEndpoint<E>(std::sync::Arc<E>);

impl<E> StripPrefixEndpoint<E> {
    fn new(ep: E) -> Self {
        Self(std::sync::Arc::new(ep))
    }
}

impl<E> Clone for StripPrefixEndpoint<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait::async_trait]
impl<State, E> Endpoint<State> for StripPrefixEndpoint<E>
where
    State: Clone + Send + Sync + 'static,
    E: Endpoint<State>,
{
    async fn call(&self, ctx: crate::Context<State>) -> crate::Result {
        let crate::Context {
            state,
            mut req,
            res,
            params,
        } = ctx;

        let rest = params
            .iter()
            .rev()
            .find_map(|captures| captures.wildcard())
            .unwrap_or_default();

        req.url_mut().set_path(rest);

        self.0
            .call(crate::Context {
                state,
                req,
                res,
                params,
            })
            .await
    }
}
