use super::resp::{serialize, unserialize, Resp};
use super::{Engine, Error, ErrorKind, Result};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

pub struct Server<E: Engine> {
    engine: E,
}

impl<E: Engine> Server<E> {
    pub fn new(engine: E) -> Self {
        Server { engine }
    }

    pub async fn run<A: ToSocketAddrs>(self, addr: A) -> Result<()> {
        // bind remote addr for tcp server
        let listener = TcpListener::bind(addr).await?;
        loop {
            listener.accept().await?;
        }

        Ok(())
    }
}
