use super::{is_transient_error, ListenInfo};

use crate::listener::Listener;
use crate::{Server, EnvoyErr};

use std::fmt::{self, Display, Formatter};

use async_std::net::{self, SocketAddr, TcpStream};
use async_std::prelude::*;
use async_std::{io, task};
use tracing::Level;

/// This represents a envoy [Listener](crate::listener::Listener) that
/// wraps an [async_std::net::TcpListener]. It is implemented as an
/// enum in order to allow creation of a envoy::listener::TcpListener
/// from a SocketAddr spec that has not yet been bound OR from a bound
/// TcpListener.
///
/// This is currently crate-visible only, and envoy users are expected
/// to create these through [ToListener](crate::ToListener) conversions.
pub struct TcpListener<State, Err> {
    addrs: Option<Vec<SocketAddr>>,
    listener: Option<net::TcpListener>,
    server: Option<Server<State, Err>>,
    info: Option<ListenInfo>,
}

impl<State, Err> TcpListener<State, Err> {
    pub fn from_addrs(addrs: Vec<SocketAddr>) -> Self {
        Self {
            addrs: Some(addrs),
            listener: None,
            server: None,
            info: None,
        }
    }

    pub fn from_listener(tcp_listener: impl Into<net::TcpListener>) -> Self {
        Self {
            addrs: None,
            listener: Some(tcp_listener.into()),
            server: None,
            info: None,
        }
    }
}

fn handle_tcp<State: Clone + Send + Sync + 'static, Err: EnvoyErr>(app: Server<State, Err>, stream: TcpStream) {
    task::spawn(async move {
        let local_addr = stream.local_addr().ok();
        let peer_addr = stream.peer_addr().ok();

        let fut = async_h1::accept(stream, |mut req| async {
            req.set_local_addr(local_addr);
            req.set_peer_addr(peer_addr);
            app.respond(req).await
        });

        if let Err(error) = fut.await {
            tracing::event!(Level::INFO, "async-h1 error {}",
                error
            );
        }
    });
}

#[async_trait::async_trait]
impl<State, Err: EnvoyErr> Listener<State, Err> for TcpListener<State, Err>
where
    State: Clone + Send + Sync + 'static,
{
    async fn bind(&mut self, server: Server<State, Err>) -> io::Result<()> {
        assert!(self.server.is_none(), "`bind` should only be called once");
        self.server = Some(server);

        if self.listener.is_none() {
            let addrs = self
                .addrs
                .take()
                .expect("`bind` should only be called once");
            let listener = net::TcpListener::bind(addrs.as_slice()).await?;
            self.listener = Some(listener);
        }

        // Format the listen information.
        let conn_string = format!("{}", self);
        let transport = "tcp".to_owned();
        let tls = false;
        self.info = Some(ListenInfo::new(conn_string, transport, tls));

        Ok(())
    }

    async fn accept(&mut self) -> io::Result<()> {
        let server = self
            .server
            .take()
            .expect("`Listener::bind` must be called before `Listener::accept`");
        let listener = self
            .listener
            .take()
            .expect("`Listener::bind` must be called before `Listener::accept`");

        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            match stream {
                Err(ref e) if is_transient_error(e) => continue,
                Err(error) => {
                    let delay = std::time::Duration::from_millis(500);
                    tracing::event!(Level::INFO, "Error: {}. Pausing for {:?}.", error, delay);
                    task::sleep(delay).await;
                    continue;
                }

                Ok(stream) => {
                    handle_tcp(server.clone(), stream);
                }
            };
        }
        Ok(())
    }

    fn info(&self) -> Vec<ListenInfo> {
        match &self.info {
            Some(info) => vec![info.clone()],
            None => vec![],
        }
    }
}

impl<State, Err> fmt::Debug for TcpListener<State, Err> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TcpListener")
            .field("listener", &self.listener)
            .field("addrs", &self.addrs)
            .field(
                "server",
                if self.server.is_some() {
                    &"Some(Server<State>)"
                } else {
                    &"None"
                },
            )
            .finish()
    }
}

impl<State, Err> Display for TcpListener<State, Err> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let http_fmt = |a| format!("http://{}", a);
        match &self.listener {
            Some(listener) => {
                let addr = listener.local_addr().expect("Could not get local addr");
                write!(f, "{}", http_fmt(&addr))
            }
            None => match &self.addrs {
                Some(addrs) => {
                    let addrs = addrs.iter().map(http_fmt).collect::<Vec<_>>().join(", ");
                    write!(f, "{}", addrs)
                }
                None => write!(f, "Not listening. Did you forget to call `Listener::bind`?"),
            },
        }
    }
}
