use crate::listener::{ListenInfo, Listener, ToListener};
use crate::{Server};

use std::fmt::{self, Debug, Display, Formatter};

use tokio::io;
use futures_util::stream::{futures_unordered::FuturesUnordered, StreamExt};

/// ConcurrentListener allows envoy to listen on any number of transports
/// simultaneously (such as tcp ports, unix sockets, or tls).


#[derive(Default)]
pub struct ConcurrentListener {
    listeners: Vec<Box<dyn Listener>>,
}

impl ConcurrentListener {
    /// creates a new ConcurrentListener
    pub fn new() -> Self {
        Self { listeners: vec![] }
    }

    pub fn add<L>(&mut self, listener: L) -> io::Result<()>
    where
        L: ToListener,
    {
        self.listeners.push(Box::new(listener.to_listener()?));
        Ok(())
    }


    pub fn with_listener<L>(mut self, listener: L) -> Self
    where
        L: ToListener,
    {
        self.add(listener).expect("Unable to add listener");
        self
    }
}

#[async_trait::async_trait]
impl Listener for ConcurrentListener
where
    {
    async fn bind(&mut self, app: Server) -> io::Result<()> {
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

impl Debug for ConcurrentListener {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.listeners)
    }
}

impl Display for ConcurrentListener {
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
