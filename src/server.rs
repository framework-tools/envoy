//! An HTTP server

use std::sync::Arc;

use tokio::io;

use crate::listener::{Listener, ToListener};
use crate::middleware::{Middleware, Next};
use crate::router::{Router, Selection};
use crate::{Endpoint, Route};

/// An HTTP server.
///
/// Servers are built up as a combination of *state*, *endpoints* and *middleware*:
///
/// - Server state is user-defined, and is provided via the [`Server::with_state`] function. The
/// state is available as a shared reference to all app endpoints.
///
/// - Endpoints provide the actual application-level code corresponding to
/// particular URLs. The [`Server::at`] method creates a new *route* (using
/// standard router syntax), which can then be used to register endpoints
/// for particular HTTP request types.
///
/// - Middleware extends the base Envoy framework with additional request or
/// response processing, such as compression, default headers, or logging. To
/// add middleware to an app, use the [`Server::with`] method.
pub struct Server {
    router: Arc<Router>,
    /// Holds the middleware stack.
    ///
    /// Note(Fishrock123): We do actually want this structure.
    /// The outer Arc allows us to clone in .respond() without cloning the array.
    /// The Vec allows us to add middleware at runtime.
    /// The inner Arc-s allow MiddlewareEndpoint-s to be cloned internally.
    /// We don't use a Mutex around the Vec here because adding a middleware during execution should be an error.
    #[allow(clippy::rc_buffer)]
    middleware: Arc<Vec<Arc<dyn Middleware>>>,
}

impl Server {
    /// Create a new Envoy server.
    #[must_use]
    pub fn new() -> Self {
        Self {
            router: Arc::new(Router::new()),
            middleware: Arc::new(Vec::new()),
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {


    /// Add a new route at the given `path`, relative to root.
    ///
    /// Routing means mapping an HTTP request to an endpoint. Here Envoy applies
    /// a "table of contents" approach, which makes it easy to see the overall
    /// app structure. Endpoints are selected solely by the path and HTTP method
    /// of a request: the path determines the resource and the HTTP verb the
    /// respective endpoint of the selected resource.
    ///
    /// A path is comprised of zero or many segments, i.e. non-empty strings
    /// separated by '/'. There are two kinds of segments: concrete and
    /// wildcard. A concrete segment is used to exactly match the respective
    /// part of the path of the incoming request. A wildcard segment on the
    /// other hand extracts and parses the respective part of the path of the
    /// incoming request to pass it along to the endpoint as an argument. A
    /// wildcard segment is written as `:name`, which creates an endpoint
    /// parameter called `name`. It is not possible to define wildcard segments
    /// with different names for otherwise identical paths.
    ///
    /// Alternatively a wildcard definitions can start with a `*`, for example
    /// `*path`, which means that the wildcard will match to the end of given
    /// path, no matter how many segments are left, even nothing.
    ///
    /// The name of the parameter can be omitted to define a path that matches
    /// the required structure, but where the parameters are not required.
    /// `:` will match a segment, and `*` will match an entire path.
    ///
    /// Here are some examples omitting the HTTP verb based endpoint selection:
    ///
    /// ```rust,no_run
    /// # let mut app = envoy::Server::new();
    /// app.at("/");
    /// app.at("/hello");
    /// app.at("add_two/:num");
    /// app.at("files/:user/*");
    /// app.at("static/*path");
    /// app.at("static/:context/:");
    /// ```
    ///
    /// There is no fallback route matching, i.e. either a resource is a full
    /// match or not, which means that the order of adding resources has no
    /// effect.
    pub fn at<'a>(&'a mut self, path: &str) -> Route<'a> {
        let router = Arc::get_mut(&mut self.router)
            .expect("Registering routes is not possible after the Server has started");
        Route::new(router, path.to_owned())
    }

    /// Add middleware to an application.
    ///
    /// Middleware provides customization of the request/response cycle, such as compression,
    /// logging, or header modification. Middleware is invoked when processing a request, and can
    /// either continue processing (possibly modifying the response) or immediately return a
    /// response. See the [`Middleware`] trait for details.
    ///
    /// Middleware can only be added at the "top level" of an application, and is processed in the
    /// order in which it is applied.
    pub fn with(&mut self, middleware: impl Middleware + 'static) -> &mut Self
    {
        tracing::trace!("Adding middleware {}", middleware.name());
        let m = Arc::get_mut(&mut self.middleware)
            .expect("Registering middleware is not possible after the Server has started");
        m.push(Arc::new(middleware));
        self
    }

    /// Asynchronously serve the app with the supplied listener.
    ///
    /// This is a shorthand for calling `Server::bind`, logging the `ListenInfo`
    /// instances from `Listener::info`, and then calling `Listener::accept`.
    pub async fn listen<L: ToListener>(self, listener: L) -> io::Result<()> {
        let mut listener = listener.to_listener()?;
        listener.bind(self).await?;
        for info in listener.info().iter() {
            tracing::info!("Server listening on {}", info);
        }
        listener.accept().await?;
        Ok(())
    }

    /// Asynchronously bind the listener.
    ///
    /// Bind the listener. This starts the listening process by opening the
    /// necessary network ports, but not yet accepting incoming connections.
    /// `Listener::listen` should be called after this to start accepting
    /// connections.
    ///
    /// When calling `Listener::info` multiple `ListenInfo` instances may be
    /// returned. This is useful when using for example `ConcurrentListener`
    /// which enables a single server to listen on muliple ports.
    pub async fn bind<L: ToListener>(
        self,
        listener: L,
    ) -> io::Result<<L as ToListener>::Listener> {
        let mut listener = listener.to_listener()?;
        listener.bind(self).await?;
        Ok(listener)
    }

    /// Respond to a `Request` with a `Response`.
    ///
    /// This method is useful for testing endpoints directly,
    /// or for creating servers over custom transports.
    pub async fn respond<Req, Res>(&self, req: Req) -> http_types::Result<Res>
    where
        Req: Into<http_types::Request>,
        Res: From<http_types::Response>,
    {
        let req = req.into();
        let Self {
            router,
            middleware,
        } = self.clone();

        let method = req.method().to_owned();
        let Selection { endpoint, params } = router.route(req.url().path(), method);
        let route_params = vec![params];
        let mut ctx = crate::Context::new(req, route_params);

        let next = Next::new(endpoint, middleware);

        if let Err(err) = next.run(&mut ctx).await {
            ctx.res.set_body(err.to_string());
            ctx.res.set_status(err.status());
        }

        Ok(ctx.res.into())
    }

}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server").finish()
    }
}

impl Clone for Server {
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
            middleware: self.middleware.clone(),
        }
    }
}

#[async_trait::async_trait]
impl Endpoint for Server
{
    async fn call(&self, ctx: &mut crate::Context) -> crate::Result {
        let path = ctx.req.url().path().to_owned();
        let method = ctx.req.method().to_owned();
        let router = self.router.clone();
        let middleware = self.middleware.clone();

        let Selection { endpoint, params } = router.route(&path, method);
        ctx.params.push(params);

        let next = Next::new(endpoint, middleware);

        next.run(ctx).await
    }
}

#[async_trait::async_trait]
impl http_client::HttpClient for Server {
    async fn send(&self, req: crate::http::Request) -> crate::http::Result<crate::http::Response> {
        self.respond(req).await
    }
}

#[cfg(test)]
mod test {
    use crate as envoy;

    #[test]
    fn allow_nested_server_with_same_state() {
        let inner = envoy::new();
        let mut outer = envoy::new();
        outer.at("/foo").get(inner);
    }
}
