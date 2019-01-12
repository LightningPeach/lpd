mod tcp;

use secp256k1::{SecretKey, PublicKey};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    prelude::{Future, Stream, Poll},
    codec::Framed,
    prelude::stream::{SplitSink, SplitStream},
};
use brontide::{BrontideStream, HandshakeError, Machine};
use futures::sync::{oneshot, mpsc};
use either::Either;
use std::collections::BTreeMap;

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
    stream: MessageStream<SplitStream<Framed<S, Machine>>>,
    identity: PublicKey,
}

impl<S> Connection<S>
where
    S: AsyncRead + AsyncWrite,
{
    fn new(brontide_stream: BrontideStream<S>, termination: oneshot::Receiver<()>, control: mpsc::UnboundedReceiver<DirectCommand>) -> Self {
        let identity = brontide_stream.remote_key();
        let (sink, stream) = brontide_stream.framed().split();
        Connection {
            sink: sink,
            stream: MessageStream {
                inner: stream,
                termination: termination,
                control: control,
            },
            identity: identity,
        }
    }

    pub fn remote_key(&self) -> PublicKey {
        self.identity.clone()
    }

    pub fn split(self) -> (SplitSink<Framed<S, Machine>>, MessageStream<SplitStream<Framed<S, Machine>>>) {
        (self.sink, self.stream)
    }
}

/// Controlled stream
pub struct MessageStream<S>
where
    S: Stream,
{
    inner: S,
    termination: oneshot::Receiver<()>,
    control: mpsc::UnboundedReceiver<DirectCommand>
}

impl<S> Stream for MessageStream<S>
where
    S: Stream,
{
    type Item = Either<S::Item, DirectCommand>;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use tokio::prelude::Async::*;

        match self.termination.poll().unwrap() {
            Ready(_) => Ok(Ready(None)),
            NotReady => {
                match self.control.poll().unwrap() {
                    Ready(Some(command)) => Ok(Ready(Some(Either::Right(command)))),
                    Ready(None) | NotReady => self.inner.poll()
                        .map(|a|
                            a.map(|maybe| maybe.map(Either::Left))
                        ),
                }
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
    pipes: BTreeMap<PublicKey, (oneshot::Sender<()>, mpsc::UnboundedSender<DirectCommand>)>,
}

impl<A, C> ConnectionStream<A, C>
where
    A: AbstractAddress,
    C: Stream<Item=Command<A>, Error=()>,
{
    pub fn listen(address: &A, control: C, local_secret: SecretKey) -> Result<Self, A::Error> {
        Ok(ConnectionStream {
            incoming: address.listen(local_secret.clone())?,
            outgoing: Vec::new(),
            control: control,
            local_secret: local_secret,
            pipes: BTreeMap::new(),
        })
    }
}

#[allow(non_shorthand_field_patterns)]
impl<A, C> Stream for ConnectionStream<A, C>
where
    A: AbstractAddress,
    C: Stream<Item=Command<A>, Error=()>,
{
    type Item = Connection<A::Stream>;
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
                    destination: destination,
                    command: command,
                } => {
                    // TODO: handle errors
                    if let Some((_, ref ctx)) = self.pipes.get(&destination) {
                        ctx.unbounded_send(command).unwrap();
                    } else {
                        // destination is not found
                    }
                    Ok(NotReady)
                },
                Command::Terminate => {
                    use std::mem;

                    let mut empty = BTreeMap::new();
                    mem::swap(&mut empty, &mut self.pipes);
                    // TODO: handle errors
                    empty.into_iter().for_each(|(_, (ttx, _))| ttx.send(()).unwrap());
                    Ok(Ready(None))
                },
            },
            NotReady => {
                match self.incoming.poll()? {
                    Ready(None) => Ok(Ready(None)),
                    Ready(Some(brontide_stream)) => {
                        let (ttx, trx) = oneshot::channel();
                        let (ctx, crx) = mpsc::unbounded();
                        self.pipes.insert(brontide_stream.remote_key(), (ttx, ctx));
                        Ok(Ready(Some(Connection::new(brontide_stream, trx, crx))))
                    },
                    NotReady => {
                        for (index, r) in self.outgoing.iter_mut().enumerate() {
                            match r.poll() {
                                Ok(NotReady) => (),
                                t @ _ => {
                                    self.outgoing.remove(index);
                                    return t.map(|a| a.map(|brontide_stream| {
                                        let (ttx, trx) = oneshot::channel();
                                        let (ctx, crx) = mpsc::unbounded();
                                        self.pipes.insert(brontide_stream.remote_key(), (ttx, ctx));
                                        Some(Connection::new(brontide_stream, trx, crx))
                                    }))
                                }
                            }
                        }
                        Ok(NotReady)
                    }
                }
            }
        }
    }
}
