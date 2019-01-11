use super::AbstractAddress;

use std::{net::SocketAddr, io};
use secp256k1::{SecretKey, PublicKey};
use tokio::{
    prelude::{Future, Stream, Poll},
    net::{TcpStream, TcpListener, tcp::{ConnectFuture, Incoming}},
};
use brontide::{BrontideStream, HandshakeError};

impl AbstractAddress for SocketAddr {
    type Error = io::Error;
    type Stream = TcpStream;
    type Outgoing = TcpConnection;
    type Incoming = TcpConnectionStream;

    fn connect(&self, local_secret: SecretKey, remote_public: PublicKey) -> Self::Outgoing {
        TcpConnection {
            inner: TcpStream::connect(self),
            handshake: None,
            local_secret: local_secret,
            remote_public: remote_public,
        }
    }

    fn listen(&self, local_secret: SecretKey) -> Result<Self::Incoming, Self::Error> {
        Ok(TcpConnectionStream {
            inner: TcpListener::bind(self).map(TcpListener::incoming)?,
            handshake: None,
            local_secret: local_secret,
        })
    }
}

pub struct TcpConnection {
    inner: ConnectFuture,
    handshake: Option<Box<dyn Future<Item=BrontideStream<TcpStream>, Error=HandshakeError> + Send + 'static>>,
    local_secret: SecretKey,
    remote_public: PublicKey,
}

impl Future for TcpConnection {
    type Item = BrontideStream<TcpStream>;
    type Error = HandshakeError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use tokio::prelude::Async::*;

        match &mut self.handshake {
            &mut None => match self.inner.poll().map_err(HandshakeError::Io)? {
                NotReady => Ok(NotReady),
                Ready(stream) => {
                    let handshake = BrontideStream::outgoing(
                        stream,
                        self.local_secret.clone(),
                        self.remote_public.clone()
                    );
                    self.handshake = Some(Box::new(handshake));
                    self.poll()
                },
            },
            &mut Some(ref mut f) => match f.poll() {
                Ok(NotReady) => Ok(NotReady),
                r @ _ => {
                    self.handshake = None;
                    r
                },
            }
        }
    }
}

pub struct TcpConnectionStream {
    inner: Incoming,
    handshake: Option<Box<dyn Future<Item=BrontideStream<TcpStream>, Error=HandshakeError> + Send + 'static>>,
    local_secret: SecretKey,
}

impl Stream for TcpConnectionStream {
    type Item = BrontideStream<TcpStream>;
    type Error = HandshakeError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use tokio::prelude::Async::*;

        match &mut self.handshake {
            &mut None => match self.inner.poll().map_err(HandshakeError::Io)? {
                NotReady => Ok(NotReady),
                Ready(None) => Ok(Ready(None)),
                Ready(Some(stream)) => {
                    let handshake = BrontideStream::incoming(stream, self.local_secret.clone());
                    self.handshake = Some(Box::new(handshake));
                    self.poll()
                },
            },
            &mut Some(ref mut f) => match f.poll() {
                Ok(NotReady) => Ok(NotReady),
                r @ _ => {
                    self.handshake = None;
                    r.map(|a| a.map(Some))
                },
            }
        }
    }
}
