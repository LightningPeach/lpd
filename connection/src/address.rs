use std::net::SocketAddr;
use secp256k1::{SecretKey, PublicKey};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    prelude::{Future, Stream, Poll},
    net::{TcpStream, TcpListener, tcp::{ConnectFuture, Incoming}},
};
use brontide::{BrontideStream, HandshakeError};

pub trait AbstractAddress {
    type Stream: AsyncRead + AsyncWrite + Send + 'static;
    type Outgoing: Future<Item=BrontideStream<Self::Stream>, Error=HandshakeError> + Send + 'static;
    type Incoming: Stream<Item=BrontideStream<Self::Stream>, Error=HandshakeError> + Send + 'static;

    fn connect(&self, local_secret: SecretKey, remote_public: PublicKey) -> Self::Outgoing;
    fn listen(&self, local_secret: SecretKey) -> Self::Incoming;
}

impl AbstractAddress for SocketAddr {
    type Stream = TcpStream;
    type Outgoing = TcpConnection;
    type Incoming = TcpConnectionStream;

    fn connect(&self, local_secret: SecretKey, remote_public: PublicKey) -> Self::Outgoing {
        TcpConnection {
            inner: TcpStream::connect(self),
            local_secret: local_secret,
            remote_public: remote_public,
        }
    }

    fn listen(&self, local_secret: SecretKey) -> Self::Incoming {
        TcpConnectionStream {
            inner: TcpListener::bind(self)
                .map_err(HandshakeError::Io)
                .map_err(Some)
                .map(TcpListener::incoming),
            local_secret: local_secret,
        }
    }
}

pub struct TcpConnection {
    inner: ConnectFuture,
    local_secret: SecretKey,
    remote_public: PublicKey,
}

impl Future for TcpConnection {
    type Item = BrontideStream<TcpStream>;
    type Error = HandshakeError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use tokio::prelude::Async::*;
        match self.inner.poll() {
            Ok(Ready(stream)) => {
                BrontideStream::outgoing(
                    stream,
                    self.local_secret.clone(),
                    self.remote_public.clone()
                ).poll()
            },
            Ok(NotReady) => Ok(NotReady),
            Err(error) => Err(HandshakeError::Io(error)),
        }
    }
}

pub struct TcpConnectionStream {
    inner: Result<Incoming, Option<HandshakeError>>,
    local_secret: SecretKey,
}

impl Stream for TcpConnectionStream {
    type Item = BrontideStream<TcpStream>;
    type Error = HandshakeError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use tokio::prelude::Async::*;
        use std::mem;

        match &mut self.inner {
            &mut Ok(ref mut inner) => match inner.poll() {
                Ok(Ready(Some(stream))) => BrontideStream::incoming(stream, self.local_secret.clone())
                    .poll().map(|a| a.map(Some)),
                Ok(Ready(None)) => Ok(Ready(None)),
                Ok(NotReady) => Ok(NotReady),
                Err(error) => Err(HandshakeError::Io(error)),
            },
            &mut Err(ref mut error) => {
                let mut temp = None;
                mem::swap(error, &mut temp);
                match temp {
                    None => Ok(Ready(None)),
                    Some(error) => Err(error),
                }
            }
        }
    }
}
