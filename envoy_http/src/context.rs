use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

use hyper::{Body};
use routefinder::Captures;

/// ## The context of a request.
///
/// This is a wrapper around a [crate::http::Request] and a [crate::http::Response]
/// that provides access to the request and response
/// as well as some additional information such as the state
/// and parameters.
#[derive(Debug)]
pub struct Context {
    state: HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>,
    /// Any error captured during the request.
    /// The parsed request parameters
    pub params: Vec<Captures<'static, 'static>>,
}

impl Context {
    /// Create a new [Context] with a [crate::http::Request].
    pub(crate) fn new(req: crate::Request<Body>, params: Vec<Captures<'static, 'static>>) -> Self {
        let mut ctx = Self {
            state: HashMap::new(),
            params,
        };

        let (
            hyper::http::request::Parts {
                method,
                uri,
                version,
                headers,
                extensions,
                ..
            },
            body,
        ) = req.into_parts();

        ctx.insert(method);
        ctx.insert(uri);
        ctx.insert(version);
        ctx.insert(headers);
        ctx.insert(body);
        ctx.insert(extensions);

        ctx
    }

    /// Try borrow a context value
    #[must_use]
    pub fn try_borrow<T: 'static>(&self) -> Option<&T> {
        self.state
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
    }

    /// Borrow a context value
    /// Panics if the value does not exist
    #[must_use]
    pub fn borrow<T: 'static>(&self) -> &T {
        self.try_borrow().unwrap_or_else(|| {
            panic!(
                "Context value `{}` does not exist",
                std::any::type_name::<T>()
            )
        })
    }

    /// Try borrow a mutable context value
    #[must_use]
    pub fn try_borrow_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.state
            .get_mut(&TypeId::of::<T>())
            .and_then(|v| v.downcast_mut::<T>())
    }

    /// Borrow a mutable context value
    /// Panics if the value does not exist
    #[must_use]
    pub fn borrow_mut<T: 'static>(&mut self) -> &mut T {
        match self.try_borrow_mut() {
            Some(v) => v,
            None => panic!(
                "Context value `{}` does not exist",
                std::any::type_name::<T>()
            ),
        }
    }

    /// Try take a context value
    #[must_use]
    pub fn try_take<T: 'static>(&mut self) -> Option<T> {
        self.state
            .remove(&TypeId::of::<T>())
            .and_then(|v| v.downcast::<T>().map(|v| *v).ok())
    }

    /// Take a context value
    /// Panics if the value does not exist
    #[must_use]
    pub fn take<T: 'static>(&mut self) -> T {
        match self.try_take() {
            Some(v) => v,
            None => panic!(
                "Context value `{}` does not exist",
                std::any::type_name::<T>()
            ),
        }
    }

    /// Insert a context value
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) -> Option<T> {
        self.state
            .insert(TypeId::of::<T>(), Box::new(value))
            .map(|v| *v.downcast::<T>().unwrap())
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
