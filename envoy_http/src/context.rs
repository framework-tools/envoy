use std::{fmt::Debug, collections::HashMap, any::{TypeId, Any}};

use hyper::{Version, Body, Method, header::{HeaderName, HeaderValue}, Uri};
use routefinder::Captures;

/// ## The context of a request.
///
/// This is a wrapper around a [crate::http::Request] and a [crate::http::Response]
/// that provides access to the request and response
/// as well as some additional information such as the state
/// and parameters.
#[derive(Debug)]
pub struct Context {

    /// The request that was made.
    pub req: crate::Request<Body>,
    ctx: HashMap<TypeId, Box<dyn Any + Send>>,
    /// Any error captured during the request.
    /// The parsed request parameters
    pub params: Vec<Captures<'static, 'static>>,
}


impl Context {
    /// Create a new [Context] with a [crate::http::Request].
    pub(crate) fn new(

        req: crate::Request<Body>,
        params: Vec<Captures<'static, 'static>>,
    ) -> Self {
        Self {
            req,
            ctx: HashMap::new(),
            params,
        }
    }

    /// Access the request's HTTP method.
    #[must_use]
    pub fn method(&self) -> &Method {
        self.req.method()
    }

    /// Access the request's full URI method.
    #[must_use]
    pub fn uri(&self) -> &Uri {
        self.req.uri()
    }

    /// Access the request's HTTP version.
    #[must_use]
    pub fn version(&self) -> Version {
        self.req.version()
    }

    /// Get the destination host for this request.
    ///
    /// This is determined in the following priority:
    /// 1. `Forwarded` header `host` key
    /// 2. The first `X-Forwarded-Host` header
    /// 3. `Host` header
    /// 4. URL domain, if any
    #[must_use]
    pub fn host(&self) -> Option<&str> {
        self.req.uri().host()
    }

    /// Get an HTTP header.
    /// ```
    #[must_use]
    pub fn header(
        &self,
        key: impl Into<HeaderName>,
    ) -> Option<&HeaderValue> {
        self.req.headers().get(key.into())
    }

    /// Get a mutable reference to a header.
    pub fn header_mut(&mut self, name: impl Into<HeaderName>) -> Option<&mut HeaderValue> {
        self.req.headers_mut().get_mut(name.into())
    }

    /// Set an HTTP header.
    pub fn insert_header(
        &mut self,
        name: impl Into<HeaderName>,
        value: impl Into<HeaderValue>,
    ) -> Option<HeaderValue> {
        self.req.headers_mut().insert(name.into(), value.into())
    }

    /// Remove a header.
    pub fn remove_header(&mut self, name: impl Into<HeaderName>) -> Option<HeaderValue> {
        self.req.headers_mut().remove(name.into())
    }

    /// Try borrow a context value
    #[must_use]
    pub fn try_borrow<T: 'static>(&self) -> Option<&T> {
        self.ctx.get(&TypeId::of::<T>()).and_then(|v| v.downcast_ref::<T>())
    }

    /// Borrow a context value
    /// Panics if the value does not exist
    #[must_use]
    pub fn borrow<T: 'static>(&self) -> &T {
        self.try_borrow().unwrap_or_else(|| {
            panic!("Context value `{}` does not exist", std::any::type_name::<T>())
        })
    }

    /// Try borrow a mutable context value
    #[must_use]
    pub fn try_borrow_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.ctx.get_mut(&TypeId::of::<T>()).and_then(|v| v.downcast_mut::<T>())
    }

    /// Borrow a mutable context value
    /// Panics if the value does not exist
    #[must_use]
    pub fn borrow_mut<T: 'static>(&mut self) -> &mut T {
        match self.try_borrow_mut() {
            Some(v) => v,
            None => panic!("Context value `{}` does not exist", std::any::type_name::<T>()),
        }
    }

    /// Try take a context value
    #[must_use]
    pub fn try_take<T: 'static>(&mut self) -> Option<T> {
        self.ctx.remove(&TypeId::of::<T>()).and_then(|v| v.downcast::<T>().map(|v| *v).ok())
    }

    /// Take a context value
    /// Panics if the value does not exist
    #[must_use]
    pub fn take<T: 'static>(&mut self) -> T {
        match self.try_take() {
            Some(v) => v,
            None => panic!("Context value `{}` does not exist", std::any::type_name::<T>()),
        }
    }

    /// Insert a context value
    pub fn insert<T: Send + 'static>(&mut self, value: T) -> Option<T> {
        self.ctx.insert(TypeId::of::<T>(), Box::new(value)).map(|v| *v.downcast::<T>().unwrap())
    }

    /// Extract and parse a route parameter by name.
    ///
    /// Returns the parameter as a `&str`, borrowed from this `Request`.
    ///
    /// The name should *not* include the leading `:`.
    ///
    /// # Errors
    ///
    /// An error is returned if `key` is not a valid parameter for the route.

    pub fn param(&self, key: &str) -> crate::Result<&str> {
        self.params
            .iter()
            .rev()
            .find_map(|captures| captures.get(key))
            .ok_or_else(|| anyhow::anyhow!("Param \"{}\" not found", key.to_string()).into())
    }

    /// Fetch the wildcard from the route, if it exists
    ///
    /// Returns the parameter as a `&str`, borrowed from this `Request`.

    pub fn wildcard(&self) -> Option<&str> {
        self.params
            .iter()
            .rev()
            .find_map(|captures| captures.wildcard())
    }
}

impl AsRef<crate::Request<Body>> for Context {
    fn as_ref(&self) -> &crate::Request<Body> {
        &self.req
    }
}

impl AsMut<crate::Request<Body>> for Context {
    fn as_mut(&mut self) -> &mut crate::Request<Body> {
        &mut self.req
    }
}