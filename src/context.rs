use routefinder::Captures;
use crate::http::headers::{HeaderName, HeaderValues, ToHeaderValues};
use crate::http::{headers, Body, Method, Mime, StatusCode, Url, Version};
use crate::http::format_err;

/// ## The context of a request.
///
/// This is a wrapper around a [crate::http::Request] and a [crate::http::Response]
/// that provides access to the request and response
/// as well as some additional information such as the state
/// and parameters.
#[derive(Debug)]
pub struct Context {

    /// The request that was made.
    pub req: crate::http::Request,
    /// The response that will be sent.
    pub res: crate::http::Response,
    /// Any error captured during the request.
    /// The parsed request parameters
    pub params: Vec<Captures<'static, 'static>>,
}


impl Context {
    /// Create a new [Context] with a [crate::http::Request].
    pub(crate) fn new(

        req: crate::http::Request,
        params: Vec<Captures<'static, 'static>>,
    ) -> Self {
        Self {
            req,
            res: crate::http::Response::new(StatusCode::Ok),
            params,
        }
    }

    /// Access the request's HTTP method.
    #[must_use]
    pub fn method(&self) -> Method {
        self.req.method()
    }

    /// Access the request's full URI method.
    #[must_use]
    pub fn url(&self) -> &Url {
        self.req.url()
    }

    /// Access the request's HTTP version.
    #[must_use]
    pub fn version(&self) -> Option<Version> {
        self.req.version()
    }

    /// Get the peer socket address for the underlying transport, if
    /// that information is available for this request.
    #[must_use]
    pub fn peer_addr(&self) -> Option<&str> {
        self.req.peer_addr()
    }

    /// Get the local socket address for the underlying transport, if
    /// that information is available for this request.
    #[must_use]
    pub fn local_addr(&self) -> Option<&str> {
        self.req.local_addr()
    }

    /// Get the remote address for this request.
    ///
    /// This is determined in the following priority:
    /// 1. `Forwarded` header `for` key
    /// 2. The first `X-Forwarded-For` header
    /// 3. Peer address of the transport
    #[must_use]
    pub fn remote(&self) -> Option<&str> {
        self.req.remote()
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
        self.req.host()
    }

    /// Get the request content type as a `Mime`.
    ///
    /// This gets the request `Content-Type` header.
    ///
    /// [Read more on MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types)
    #[must_use]
    pub fn content_type(&self) -> Option<Mime> {
        self.req.content_type()
    }

    /// Get an HTTP header.
    /// ```
    #[must_use]
    pub fn header(
        &self,
        key: impl Into<http_types::headers::HeaderName>,
    ) -> Option<&http_types::headers::HeaderValues> {
        self.req.header(key)
    }

    /// Get a mutable reference to a header.
    pub fn header_mut(&mut self, name: impl Into<HeaderName>) -> Option<&mut HeaderValues> {
        self.req.header_mut(name)
    }

    /// Set an HTTP header.
    pub fn insert_header(
        &mut self,
        name: impl Into<HeaderName>,
        values: impl ToHeaderValues,
    ) -> Option<HeaderValues> {
        self.req.insert_header(name, values)
    }

    /// Append a header to the headers.
    ///
    /// Unlike `insert` this function will not override the contents of a header, but insert a
    /// header if there aren't any. Or else append to the existing list of headers.
    pub fn append_header(&mut self, name: impl Into<HeaderName>, values: impl ToHeaderValues) {
        self.req.append_header(name, values)
    }

    /// Remove a header.
    pub fn remove_header(&mut self, name: impl Into<HeaderName>) -> Option<HeaderValues> {
        self.req.remove_header(name)
    }

    /// An iterator visiting all header pairs in arbitrary order.
    #[must_use]
    pub fn iter(&self) -> headers::Iter<'_> {
        self.req.iter()
    }

    /// An iterator visiting all header pairs in arbitrary order, with mutable references to the
    /// values.
    #[must_use]
    pub fn iter_mut(&mut self) -> headers::IterMut<'_> {
        self.req.iter_mut()
    }

    /// An iterator visiting all header names in arbitrary order.
    #[must_use]
    pub fn header_names(&self) -> headers::Names<'_> {
        self.req.header_names()
    }

    /// An iterator visiting all header values in arbitrary order.
    #[must_use]
    pub fn header_values(&self) -> headers::Values<'_> {
        self.req.header_values()
    }

    /// Get a request extension value.
    #[must_use]
    pub fn ext<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.req.ext().get()
    }

    /// Get a mutable reference to value stored in request extensions.
    #[must_use]
    pub fn ext_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.req.ext_mut().get_mut()
    }

    /// Set a request extension value.
    pub fn set_ext<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.req.ext_mut().insert(val)
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
            .ok_or_else(|| format_err!("Param \"{}\" not found", key.to_string()))
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

    /// Parse the URL query component into a struct, using [serde_qs](https://docs.rs/serde_qs). To
    /// get the entire query as an unparsed string, use `request.url().query()`.

    pub fn query<'de, T: serde::de::Deserialize<'de>>(&'de self) -> crate::Result<T> {
        self.req.query()
    }

    /// Set the body reader.
    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.res.set_body(body)
    }

    /// Take the request body as a `Body`.
    ///
    /// This method can be called after the body has already been taken or read,
    /// but will return an empty `Body`.
    ///
    /// This is useful for consuming the body via an AsyncReader or AsyncBufReader.
    pub fn take_body(&mut self) -> Body {
        self.req.take_body()
    }

    /// Reads the entire request body into a byte buffer.
    ///
    /// This method can be called after the body has already been read, but will
    /// produce an empty buffer.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered while reading the body is immediately returned
    /// as an `Err`.
    pub async fn body_bytes(&mut self) -> crate::Result<Vec<u8>> {
        let res = self.req.body_bytes().await?;
        Ok(res)
    }

    /// Reads the entire request body into a string.
    ///
    /// This method can be called after the body has already been read, but will
    /// produce an empty buffer.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered while reading the body is immediately returned
    /// as an `Err`.
    ///
    /// If the body cannot be interpreted as valid UTF-8, an `Err` is returned.

    pub async fn body_string(&mut self) -> crate::Result<String> {
        let res = self.req.body_string().await?;
        Ok(res)
    }

    /// Reads and deserialized the entire request body via json.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered while reading the body is immediately returned
    /// as an `Err`.
    ///
    /// If the body cannot be interpreted as valid json for the target type `T`,
    /// an `Err` is returned.
    pub async fn body_json<T: serde::de::DeserializeOwned>(&mut self) -> crate::Result<T> {
        let res = self.req.body_json().await?;
        Ok(res)
    }

    /// Get the length of the body stream, if it has been set.
    ///
    /// This value is set when passing a fixed-size object into as the body. E.g. a string, or a
    /// buffer. Consumers of this API should check this value to decide whether to use `Chunked`
    /// encoding, or set the response length.
    #[must_use]
    pub fn len(&self) -> Option<usize> {
        self.req.len()
    }

    /// Returns `true` if the request has a set body stream length of zero, `false` otherwise.
    #[must_use]
    pub fn is_empty(&self) -> Option<bool> {
        Some(self.req.len()? == 0)
    }
}

impl AsRef<crate::http::Request> for Context {
    fn as_ref(&self) -> &crate::http::Request {
        &self.req
    }
}

impl AsMut<crate::http::Request> for Context {
    fn as_mut(&mut self) -> &mut crate::http::Request {
        &mut self.req
    }
}

impl AsRef<crate::http::Headers> for Context {
    fn as_ref(&self) -> &crate::http::Headers {
        self.req.as_ref()
    }
}

impl AsMut<crate::http::Headers> for Context {
    fn as_mut(&mut self) -> &mut crate::http::Headers {
        self.req.as_mut()
    }
}