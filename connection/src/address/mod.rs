mod tcp;

use dependencies::secp256k1;
use dependencies::tokio;
use dependencies::futures;
use dependencies::either;

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
use internal_event::{Event, DirectCommand, ChannelCommand};
use std::fmt::Formatter;
use wire::ChannelId;


pub enum TransportError {
    IOError {
        inner: std::io::Error,
        description: String,
    },
    HandshakeError {
        inner: HandshakeError,
        description: String,
    },
    TransportError {
        inner: Box<TransportError>,
        description: String,
    },
    Other {
        description: String
    }
}

impl std::fmt::Debug for TransportError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            TransportError::IOError{inner, description} => f
                .debug_struct("IOError")
                .field("inner", inner)
                .field("description", description)
                .finish(),
            TransportError::HandshakeError {inner, description} => f
                .debug_struct("HandshakeError")
                .field("inner", inner)
                .field("description", description)
                .finish(),
            TransportError::TransportError {inner, description} => f
                .debug_struct("TransportError")
                .field("inner", inner)
                .field("description", description)
                .finish(),
            TransportError::Other {description} => f
                .debug_struct("Other")
                .field("description", description)
                .finish(),
        }
    }
}

// Represent address to which we can connect
pub trait AbstractAddress : std::fmt::Debug {

    // Inner stream type used for Brontide streams
    type Stream: AsyncRead + AsyncWrite + Send + 'static;

    // Represent outgoing connection. When we try to connect to someone
    type OutgoingConnection: Future<Item=BrontideStream<Self::Stream>, Error=TransportError> + Send + 'static;

    // Represent incoming connections.
    type IncomingConnectionsStream: Stream<Item=BrontideStream<Self::Stream>, Error=TransportError> + Send + 'static;

    // Connect to remote host
    // local_secret_key - our secret_key
    // remote_public_key - remote public key
    fn connect(&self, local_secret_key: SecretKey, remote_public_key: PublicKey) -> Self::OutgoingConnection;

    // Listen for connections
    fn listen(&self, local_secret_key: SecretKey) -> Result<Self::IncomingConnectionsStream, TransportError>;
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
    fn new(brontide_stream: BrontideStream<S>, termination: oneshot::Receiver<()>, control: mpsc::UnboundedReceiver<Event>) -> Self {
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
    control: mpsc::UnboundedReceiver<Event>
}

impl<S> Stream for MessageStream<S>
where
    S: Stream,
{
    type Item = Either<S::Item, Event>;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use tokio::prelude::Async::*;

        match self.termination.poll() {
            Ok(Ready(_)) => Ok(Ready(None)),
            Ok(NotReady) => {
                // TODO(mkl): is unwrap() ok here?
                match self.control.poll().unwrap() {
                    Ready(Some(command)) => Ok(Ready(Some(Either::Right(command)))),
                    Ready(None) | NotReady => self.inner.poll()
                        .map(|a|
                            a.map(|maybe| maybe.map(Either::Left))
                        ),
                }
            }
            // TODO(mkl): why it is called
            Err(_err) => {
                // We got here when sender of self.termination is dropped
                println!("self.termination.poll() is canceled");
                Ok(Ready(None))
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
    ChannelCommand {
        destination: ChannelId,
        command: ChannelCommand,
    },
    BroadcastTick,
    Terminate,
}

pub struct ConnectionStream<A, C>
where
    A: AbstractAddress,
{
    incoming: A::IncomingConnectionsStream,
    outgoing: Vec<A::OutgoingConnection>,
    control: C,
    local_secret: SecretKey,
    pipes: BTreeMap<PublicKey, (oneshot::Sender<()>, mpsc::UnboundedSender<Event>)>,
}

impl<A, C> ConnectionStream<A, C>
where
    A: AbstractAddress,
    C: Stream<Item=Command<A>, Error=()>,
{
    pub fn listen(address: &A, control: C, local_secret: SecretKey) -> Result<Self, TransportError> {
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
    type Error = TransportError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use tokio::prelude::Async::*;

        // TODO(mkl): is it ok to unwrap here?
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
                    self.pipes.get(&destination)
                        .map(|(_, ref ctx)|
                            ctx.unbounded_send(Event::DirectCommand(command)).unwrap()
                        );
                    Ok(NotReady)
                },
                Command::ChannelCommand {
                    destination: channel_id,
                    command: command,
                } => {
                    // TODO: send the command to proper processor
                    let _ = (channel_id, command);
                    unimplemented!()
                },
                Command::BroadcastTick => {
                    self.pipes.values()
                        .for_each(|(_, ref ctx)|
                            ctx.unbounded_send(Event::TimerTick).unwrap()
                        );
                    Ok(NotReady)
                },
                Command::Terminate => {
                    use std::mem;
                    println!("Terminate connections");
                    let mut empty = BTreeMap::new();
                    mem::swap(&mut empty, &mut self.pipes);
                    empty.into_iter()
                        .for_each(|(_, (ttx, _))| match ttx.send(()) {
                            Ok(()) => (),
                            // error means that remote client is shutdown before the lpd
                            // need not to handle
                            Err(()) => (),
                        });
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
