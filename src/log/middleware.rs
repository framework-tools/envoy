use crate::{log, State};
use crate::{Middleware, Next, Request};

/// Log all incoming requests and responses.
///
/// This middleware is enabled by default in Tide. In the case of
/// nested applications, this middleware will only run once for each
/// request.
///
/// # Examples
///
/// ```
/// let mut app = tide::Server::new();
/// app.with(tide::log::LogMiddleware::new());
/// ```
#[derive(Debug, Default, Clone)]
pub struct LogMiddleware {
    _priv: (),
}

struct LogMiddlewareHasBeenRun;

impl LogMiddleware {
    /// Create a new instance of `LogMiddleware`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Log a request and a response.
    async fn log<'a, ServerState: Clone + Send + Sync + 'static>(
        &'a self,
        mut req: Request,
        state: State<ServerState>,
        next: Next<'a, ServerState>,
    ) -> crate::Result {
        if req.ext::<LogMiddlewareHasBeenRun>().is_some() {
            return Ok(next.run(req, state).await);
        }
        req.set_ext(LogMiddlewareHasBeenRun);

        let path = req.url().path().to_owned();
        let method = req.method().to_string();
        log::info!("<-- Request received", {
            method: method,
            path: path,
        });
        let start = std::time::Instant::now();
        let response = next.run(req, state).await;
        let status = response.status();
        if status.is_server_error() {
            if let Some(error) = response.error() {
                log::error!("Internal error --> Response sent", {
                    message: format!("\"{}\"", error.to_string()),
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            } else {
                log::error!("Internal error --> Response sent", {
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            }
        } else if status.is_client_error() {
            if let Some(error) = response.error() {
                log::warn!("Client error --> Response sent", {
                    message: format!("\"{}\"", error.to_string()),
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            } else {
                log::warn!("Client error --> Response sent", {
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            }
        } else {
            log::info!("--> Response sent", {
                method: method,
                path: path,
                status: format!("{} - {}", status as u16, status.canonical_reason()),
                duration: format!("{:?}", start.elapsed()),
            });
        }
        Ok(response)
    }
}

#[async_trait::async_trait]
impl<ServerState: Clone + Send + Sync + 'static> Middleware<ServerState> for LogMiddleware {
    async fn handle(
        &self,
        req: Request,
        state: State<ServerState>,
        next: Next<'_, ServerState>,
    ) -> crate::Result {
        self.log(req, state, next).await
    }
}
