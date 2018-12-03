use secp256k1::{SecretKey, PublicKey};
use std::net::SocketAddr;
use tokio::{io::{AsyncRead, AsyncWrite}, prelude::{Future, Stream, Poll}, net::{TcpStream, TcpListener}, net::tcp::{ConnectFuture, Incoming}};
use brontide::{BrontideStream, HandshakeError};

pub struct Node {
    secret: SecretKey,
}

pub trait AbstractNode<A> {
    type Stream: AsyncRead + AsyncWrite;
    type Outgoing: Future<Item=BrontideStream<Self::Stream>, Error=HandshakeError>;
    type Incoming: Stream<Item=BrontideStream<Self::Stream>, Error=HandshakeError>;

    fn connect(&self, remote_address: &A, remote_public: PublicKey) -> Self::Outgoing;
    fn listen(&self, local_address: &A) -> Self::Incoming;
}

impl AbstractNode<SocketAddr> for Node {
    type Stream = TcpStream;
    type Outgoing = TcpConnection;
    type Incoming = TcpConnectionStream;

    fn connect(&self, remote_address: &SocketAddr, remote_public: PublicKey) -> Self::Outgoing {
        TcpConnection {
            inner: TcpStream::connect(remote_address),
            local_secret: self.secret.clone(),
            remote_public: remote_public,
        }
    }

    fn listen(&self, local_address: &SocketAddr) -> Self::Incoming {
        TcpConnectionStream {
            inner: TcpListener::bind(local_address)
                .map_err(HandshakeError::Io)
                .map_err(Some)
                .map(TcpListener::incoming),
            local_secret: self.secret.clone(),
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
