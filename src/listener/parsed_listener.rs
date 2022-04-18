#[cfg(unix)]
use super::UnixListener;
use super::{ListenInfo, Listener, TcpListener};
use crate::{Server, EnvoyErr};

use async_std::io;
use std::fmt::{self, Debug, Display, Formatter};

/// This is an enum that contains variants for each of the listeners
/// that can be parsed from a string. This is used as the associated
/// Listener type for the string-parsing
/// [ToListener](crate::listener::ToListener) implementations
///
/// This is currently crate-visible only, and envoy users are expected
/// to create these through [ToListener](crate::ToListener) conversions.
pub enum ParsedListener<State, Err> {
    #[cfg(unix)]
    Unix(UnixListener<State, Err>),
    Tcp(TcpListener<State, Err>),
}

impl<State, Err> Debug for ParsedListener<State, Err> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(unix)]
            ParsedListener::Unix(unix) => Debug::fmt(unix, f),
            ParsedListener::Tcp(tcp) => Debug::fmt(tcp, f),
        }
    }
}

impl<State, Err> Display for ParsedListener<State, Err> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => write!(f, "{}", u),
            Self::Tcp(t) => write!(f, "{}", t),
        }
    }
}

#[async_trait::async_trait]
impl<State, Err: EnvoyErr> Listener<State, Err> for ParsedListener<State, Err>
where
    State: Clone + Send + Sync + 'static,
{
    async fn bind(&mut self, server: Server<State, Err>) -> io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => u.bind(server).await,
            Self::Tcp(t) => t.bind(server).await,
        }
    }

    async fn accept(&mut self) -> io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => u.accept().await,
            Self::Tcp(t) => t.accept().await,
        }
    }

    fn info(&self) -> Vec<ListenInfo> {
        match self {
            #[cfg(unix)]
            ParsedListener::Unix(unix) => unix.info(),
            ParsedListener::Tcp(tcp) => tcp.info(),
        }
    }
}
