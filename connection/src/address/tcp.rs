use super::AbstractAddress;

use std::{net::SocketAddr, io};
use secp256k1::{SecretKey, PublicKey};
use tokio::{
    prelude::{Future, Stream, Poll},
    net::{TcpStream, TcpListener, tcp::{ConnectFuture, Incoming}},
};
use brontide::{BrontideStream, HandshakeError};
use crate::address::TransportError;

impl AbstractAddress for SocketAddr {
    type Stream = TcpStream;
    type OutgoingConnection = TcpConnection;
    type IncomingConnectionsStream = TcpConnectionStream;

    fn connect(&self, local_secret: SecretKey, remote_public: PublicKey) -> Self::OutgoingConnection {
        TcpConnection {
            inner: TcpStream::connect(self),
            handshake: None,
            local_secret: local_secret,
            remote_public: remote_public,
        }
    }

    fn listen(&self, local_secret: SecretKey) -> Result<TcpConnectionStream, TransportError>
    {
        // TODO(mkl): refactor this
        let listener = TcpListener::bind(self)
            .map_err(|err| {
                TransportError::IOError {
                    inner: err,
                    description: format!("cannot create tcp listener for {:?}", self)
                }
            })?;
        let incoming_connections = listener
            .incoming()
            .map(|conn| {
                println!("incoming connection from: {:?}", conn.peer_addr());
                conn
            });
        Ok(TcpConnectionStream {
            inner: Box::new(incoming_connections),
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
    type Error = TransportError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use tokio::prelude::Async::*;
        // TODO(mkl): rewrite this
        match &mut self.handshake {
            &mut None => {
                let inner_poll = self.inner
                    .poll()
                    .map_err(|err|
                        TransportError::HandshakeError {
                            inner: HandshakeError::Io(err,"error polling inside TcpConnection".to_owned()),
                            description: "error polling inside TcpConnection".to_owned()
                        }
                    )?;
                match inner_poll {
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
                }
            },
            &mut Some(ref mut f) => {
                match f.poll() {
                    Ok(NotReady) => Ok(NotReady),
                    r @ _ => {
                        self.handshake = None;
                        r.map_err(|err|{
                            TransportError::HandshakeError {
                                inner: err,
                                description: "error polling from inner from succesfull TcpConnection poll".to_owned()
                            }
                        })
                    },
                }
            }
        }
    }
}

pub struct TcpConnectionStream
{
    inner: Box<Stream<Item=TcpStream, Error=io::Error> + Send + Sync>,
    handshake: Option<Box<dyn Future<Item=BrontideStream<TcpStream>, Error=HandshakeError> + Send + 'static>>,
    local_secret: SecretKey,
}

impl Stream for TcpConnectionStream {
    type Item = BrontideStream<TcpStream>;
    type Error = TransportError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use tokio::prelude::Async::*;
        dbg!(self.handshake.is_none());
        // TODO(mkl): rewrite this
        match &mut self.handshake {
            &mut None => {
                let inner_poll = self.inner
                    .poll()
                    .map_err(|err| TransportError::HandshakeError{
                        inner: HandshakeError::Io(err, "error polling inside TcpConnectionStream".to_owned()),
                        description: "error polling inside TcpConnectionStream".to_owned(),
                    })?;
                match inner_poll {
                    NotReady => Ok(NotReady),
                    Ready(None) => Ok(Ready(None)),
                    Ready(Some(stream)) => {
                        dbg!("TcpConnectionStream::poll before handshake init");
                        let handshake = BrontideStream::incoming(stream, self.local_secret.clone());
                        self.handshake = Some(Box::new(handshake));
                        self.poll()
                    },
                }
            },
            &mut Some(ref mut f) => {
                match f.poll() {
                    Ok(NotReady) => Ok(NotReady),
                    r @ _ => {
                        println!("WWW self.handshake = None");
                        self.handshake = None;
                        r.map(|a| a.map(Some))
                            .map_err(|err| {
                                TransportError::HandshakeError {
                                    inner: err,
                                    description: "error during inner poll during succesfull poll in TcpConnectionStream".to_owned()
                                }
                            })
                    },
                }
            }
        }
    }
}
