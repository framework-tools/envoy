use crate::listener::{ListenInfo, Listener, ToListener};
use crate::{Server, EnvoyErr};

use std::fmt::{self, Debug, Display, Formatter};

use async_std::io;
use futures_util::stream::{futures_unordered::FuturesUnordered, StreamExt};

/// ConcurrentListener allows envoy to listen on any number of transports
/// simultaneously (such as tcp ports, unix sockets, or tls).
///
/// # Example:
/// ```rust
/// fn main() -> Result<(), std::io::Error> {
///    async_std::task::block_on(async {
///        envoy::log::start();
///        let mut app = envoy::new();
///        app.at("/").get(|_| async { Ok("Hello, world!") });
///
///        let mut listener = envoy::listener::ConcurrentListener::new();
///        listener.add("127.0.0.1:8000")?;
///        listener.add(async_std::net::TcpListener::bind("127.0.0.1:8001").await?)?;
/// # if cfg!(unix) {
///        listener.add("http+unix://unix.socket")?;
/// # }
///
/// # if false {
///        app.listen(listener).await?;
/// # }
///        Ok(())
///    })
///}
///```

#[derive(Default)]
pub struct ConcurrentListener<State, Err> {
    listeners: Vec<Box<dyn Listener<State, Err>>>,
}

impl<State: Clone + Send + Sync + 'static, Err> ConcurrentListener<State, Err> {
    /// creates a new ConcurrentListener
    pub fn new() -> Self {
        Self { listeners: vec![] }
    }

    /// Adds any [`ToListener`](crate::listener::ToListener) to this
    /// ConcurrentListener. An error result represents a failure to convert
    /// the [`ToListener`](crate::listener::ToListener) into a
    /// [`Listener`](crate::listener::Listener).
    ///
    /// ```rust
    /// # fn main() -> std::io::Result<()> {
    /// let mut listener = envoy::listener::ConcurrentListener::new();
    /// listener.add("127.0.0.1:8000")?;
    /// listener.add(("localhost", 8001))?;
    /// listener.add(std::net::TcpListener::bind(("localhost", 8002))?)?;
    /// # std::mem::drop(envoy::new().listen(listener)); // for the State generic
    /// # Ok(()) }
    /// ```
    pub fn add<L>(&mut self, listener: L) -> io::Result<()>
    where
        L: ToListener<State, Err>,
    {
        self.listeners.push(Box::new(listener.to_listener()?));
        Ok(())
    }

    /// `ConcurrentListener::with_listener` allows for chained construction of a ConcurrentListener:
    /// ```rust,no_run
    /// # use envoy::listener::ConcurrentListener;
    /// # fn main() -> std::io::Result<()> { async_std::task::block_on(async move {
    /// # let app = envoy::new();
    /// app.listen(
    ///     ConcurrentListener::new()
    ///         .with_listener("127.0.0.1:8080")
    ///         .with_listener(async_std::net::TcpListener::bind("127.0.0.1:8081").await?),
    /// ).await?;
    /// #  Ok(()) }) }
    pub fn with_listener<L>(mut self, listener: L) -> Self
    where
        L: ToListener<State, Err>,
    {
        self.add(listener).expect("Unable to add listener");
        self
    }
}

#[async_trait::async_trait]
impl<State, Err: EnvoyErr> Listener<State, Err> for ConcurrentListener<State, Err>
where
    State: Clone + Send + Sync + 'static,
{
    async fn bind(&mut self, app: Server<State, Err>) -> io::Result<()> {
        for listener in self.listeners.iter_mut() {
            listener.bind(app.clone()).await?;
        }
        Ok(())
    }

    async fn accept(&mut self) -> io::Result<()> {
        let mut futures_unordered = FuturesUnordered::new();

        for listener in self.listeners.iter_mut() {
            futures_unordered.push(listener.accept());
        }

        while let Some(result) = futures_unordered.next().await {
            result?;
        }
        Ok(())
    }

    fn info(&self) -> Vec<ListenInfo> {
        self.listeners
            .iter()
            .map(|listener| listener.info().into_iter())
            .flatten()
            .collect()
    }
}

impl<State, Err> Debug for ConcurrentListener<State, Err> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.listeners)
    }
}

impl<State, Err> Display for ConcurrentListener<State, Err> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = self
            .listeners
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(f, "{}", string)
    }
}
