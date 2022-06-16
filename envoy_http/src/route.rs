use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use hyper::Uri;

use crate::endpoint::MiddlewareEndpoint;
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
pub struct Route<'a> {
    router: &'a mut Router,
    path: String,
    middleware: Vec<Arc<dyn Middleware>>,
    /// Indicates whether the path of current route is treated as a prefix. Set by
    /// [`strip_prefix`].
    ///
    /// [`strip_prefix`]: #method.strip_prefix
    prefix: bool,
}

impl<'a> Route<'a> {
    pub(crate) fn new(router: &'a mut Router, path: String) -> Route<'a> {
        Route {
            router,
            path,
            middleware: Vec::new(),
            prefix: false,
        }
    }

    /// Extend the route with the given `path`.
    pub fn at<'b>(&'b mut self, path: &str) -> Route<'b> {
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

    /// Apply the given middleware to the current route.
    pub fn with(&mut self, middleware: impl Middleware + 'static) -> &mut Self
    {
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
    /// [`Server`]: struct.Server.html
    pub fn nest(&mut self, service: crate::Server) -> &mut Self {
        let prefix = self.prefix;

        self.prefix = true;
        self.all(service);
        self.prefix = prefix;

        self
    }

    /// Add an endpoint for the given HTTP method
    pub fn method(&mut self, method: hyper::Method, ep: impl Endpoint + 'static) -> &mut Self {
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
    pub fn all(&mut self, ep: impl Endpoint + 'static) -> &mut Self {
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
    pub fn get(&mut self, ep: impl Endpoint + 'static) -> &mut Self {
        self.method(hyper::Method::GET, ep);
        self
    }

    /// Add an endpoint for `HEAD` requests
    pub fn head(&mut self, ep: impl Endpoint + 'static) -> &mut Self {
        self.method(hyper::Method::HEAD, ep);
        self
    }

    /// Add an endpoint for `PUT` requests
    pub fn put(&mut self, ep: impl Endpoint + 'static) -> &mut Self {
        self.method(hyper::Method::PUT, ep);
        self
    }

    /// Add an endpoint for `POST` requests
    pub fn post(&mut self, ep: impl Endpoint + 'static) -> &mut Self {
        self.method(hyper::Method::POST, ep);
        self
    }

    /// Add an endpoint for `DELETE` requests
    pub fn delete(&mut self, ep: impl Endpoint + 'static) -> &mut Self {
        self.method(hyper::Method::DELETE, ep);
        self
    }

    /// Add an endpoint for `OPTIONS` requests
    pub fn options(&mut self, ep: impl Endpoint + 'static) -> &mut Self {
        self.method(hyper::Method::OPTIONS, ep);
        self
    }

    /// Add an endpoint for `CONNECT` requests
    pub fn connect(&mut self, ep: impl Endpoint + 'static) -> &mut Self {
        self.method(hyper::Method::CONNECT, ep);
        self
    }

    /// Add an endpoint for `PATCH` requests
    pub fn patch(&mut self, ep: impl Endpoint + 'static) -> &mut Self {
        self.method(hyper::Method::PATCH, ep);
        self
    }

    /// Add an endpoint for `TRACE` requests
    pub fn trace(&mut self, ep: impl Endpoint + 'static) -> &mut Self {
        self.method(hyper::Method::TRACE, ep);
        self
    }
}

#[derive(Debug)]
struct StripPrefixEndpoint(std::sync::Arc<dyn Endpoint>);

impl StripPrefixEndpoint {
    fn new(ep: impl Endpoint + 'static) -> Self {
        Self(std::sync::Arc::new(ep))
    }
}

impl Clone for StripPrefixEndpoint {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait::async_trait]
impl Endpoint for StripPrefixEndpoint

{
    async fn call(&self, ctx: &mut crate::Context) -> crate::Result {

        let rest = ctx.params
            .iter()
            .rev()
            .find_map(|captures| captures.wildcard())
            .unwrap_or_default();

        *ctx.req.uri_mut() = Uri::from_str(rest)
            .map_err(|err| anyhow::anyhow!("InvalidUri: {:#?}", err))?;

        self.0
            .call(ctx)
            .await
    }
}
