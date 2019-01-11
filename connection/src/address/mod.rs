mod tcp;

use secp256k1::{SecretKey, PublicKey};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    prelude::{Future, Stream, Poll},
    codec::Framed,
    prelude::stream::{SplitSink, SplitStream},
};
use brontide::{BrontideStream, HandshakeError, Machine};

pub trait AbstractAddress {
    type Error;
    type Stream: AsyncRead + AsyncWrite + Send + 'static;
    type Outgoing: Future<Item=BrontideStream<Self::Stream>, Error=HandshakeError> + Send + 'static;
    type Incoming: Stream<Item=BrontideStream<Self::Stream>, Error=HandshakeError> + Send + 'static;

    fn connect(&self, local_secret: SecretKey, remote_public: PublicKey) -> Self::Outgoing;
    fn listen(&self, local_secret: SecretKey) -> Result<Self::Incoming, Self::Error>;
}

pub struct Connection<S>
where
    S: AsyncRead + AsyncWrite,
{
    sink: SplitSink<Framed<S, Machine>>,
    stream: MessageStream<SplitStream<Framed<S, Machine>>, Box<dyn Future<Item=(), Error=()> + Send + 'static>>,
    identity: PublicKey,
}

impl<S> Connection<S>
where
    S: Stream + AsyncRead + AsyncWrite,
{
    pub fn new<T>(brontide_stream: BrontideStream<S>, termination: T) -> Self
    where
        T: Future<Item=(), Error=()> + Send + 'static,
    {
        let identity = brontide_stream.remote_key().clone();
        let (sink, stream) = brontide_stream.framed().split();
        Connection {
            sink: sink,
            stream: MessageStream {
                inner: stream,
                control: Box::new(termination),
            },
            identity: identity,
        }
    }
}

/// Controlled stream
pub struct MessageStream<S, C>
where
    S: Stream,
{
    inner: S,
    control: C,
}

impl<S, C> Stream for MessageStream<S, C>
where
    S: Stream,
    C: Stream<Item=(), Error=()>,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use tokio::prelude::Async::*;

        match self.control.poll().unwrap() {
            Ready(_) => Ok(Ready(None)),
            NotReady => {
                self.inner.poll()
            }
        }
    }
}

pub enum Command<A>
where
    A: AbstractAddress,
{
    Connect {
        address: A,
        remote_public: PublicKey,
    },
    DirectCommand {
        destination: PublicKey,
        command: DirectCommand,
    },
    Terminate,
}

pub enum DirectCommand {
    _Nothing,
}

pub struct ConnectionStream<A, C>
where
    A: AbstractAddress,
{
    incoming: A::Incoming,
    outgoing: Vec<A::Outgoing>,
    control: C,
    local_secret: SecretKey,
}

impl<A, C> ConnectionStream<A, C>
where
    A: AbstractAddress,
    C: Stream<Item=Command<A>, Error=()>,
{
    pub fn new(address: &A, control: C, local_secret: SecretKey) -> Result<Self, A::Error> {
        Ok(ConnectionStream {
            incoming: address.listen(local_secret.clone())?,
            outgoing: Vec::new(),
            control: control,
            local_secret: local_secret,
        })
    }
}

#[allow(non_shorthand_field_patterns)]
impl<A, C> Stream for ConnectionStream<A, C>
where
    A: AbstractAddress,
    C: Stream<Item=Command<A>, Error=()>,
{
    type Item = BrontideStream<A::Stream>;
    type Error = HandshakeError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use tokio::prelude::Async::*;

        match self.control.poll().unwrap() {
            Ready(None) => Ok(Ready(None)),
            Ready(Some(command)) => match command {
                Command::Connect {
                    address: address,
                    remote_public: remote_public,
                } => {
                    let secret = self.local_secret.clone();
                    self.outgoing.push(address.connect(secret, remote_public));
                    self.poll()
                },
                Command::DirectCommand {
                    destination: _,
                    command: _,
                } => Ok(NotReady),
                Command::Terminate => Ok(Ready(None)),
            },
            NotReady => {
                let incoming = self.incoming.poll()?;
                if let Ready(t) = incoming {
                    Ok(Ready(t))
                } else {
                    for (index, r) in self.outgoing.iter_mut().enumerate() {
                        match r.poll() {
                            Ok(NotReady) => (),
                            t @ _ => {
                                self.outgoing.remove(index);
                                return t.map(|a| a.map(Some))
                            }
                        }
                    }
                    Ok(NotReady)
                }
            }
        }
    }
}
