use super::{ListenInfo, Listener, TcpListener};
use crate::{Server};

use tokio::io;
use std::fmt::{self, Debug, Display, Formatter};

/// This is an enum that contains variants for each of the listeners
/// that can be parsed from a string. This is used as the associated
/// Listener type for the string-parsing
/// [ToListener](crate::listener::ToListener) implementations
///
/// This is currently crate-visible only, and envoy users are expected
/// to create these through [ToListener](crate::ToListener) conversions.
pub enum ParsedListener {
    Tcp(TcpListener),
}

impl Debug for ParsedListener {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParsedListener::Tcp(tcp) => Debug::fmt(tcp, f),
        }
    }
}

impl Display for ParsedListener {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp(t) => write!(f, "{}", t),
        }
    }
}

#[async_trait::async_trait]
impl Listener for ParsedListener
where
    {
    async fn bind(&mut self, server: Server) -> io::Result<()> {
        match self {
            Self::Tcp(t) => t.bind(server).await,
        }
    }

    async fn accept(&mut self) -> io::Result<()> {
        match self {
            Self::Tcp(t) => t.accept().await,
        }
    }

    fn info(&self) -> Vec<ListenInfo> {
        match self {
            ParsedListener::Tcp(tcp) => tcp.info(),
        }
    }
}
