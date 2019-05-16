#![forbid(unsafe_code)]

use binformat::WireError;
use wire::{Message, MessageExt};
use internal_event::Event;

use tokio::prelude::{Future, Sink, Poll};
use futures::sink;
use either::Either;

pub trait MessageFiltered
where
    Self: Sized,
{
    fn filter(v: MessageExt) -> Result<Self, MessageExt>;
}

impl MessageFiltered for MessageExt {
    fn filter(v: MessageExt) -> Result<Self, MessageExt> {
        Ok(v)
    }
}

impl MessageFiltered for Message {
    fn filter(v: MessageExt) -> Result<Self, MessageExt> {
        Ok(v.message)
    }
}

pub trait RelevantEvent
where
    Self: Sized,
{
    fn filter(v: Event) -> Result<Self, Event>;
}

// unit means any event is irrelevant, change `()` to `!` when it stabilized
impl RelevantEvent for () {
    fn filter(v: Event) -> Result<Self, Event> {
        Err(v)
    }
}

pub enum MessageConsumerType {
    SingleResponse,
    MultipleResponse,
}

pub struct ConsumingFuture<C, S>(Box<dyn Future<Item=(C, S), Error=WireError> + Send + 'static>)
where
    S: Sink<SinkItem=MessageExt, SinkError=WireError> + Send + 'static,
    C: Send + 'static;

impl<C, S> Future for ConsumingFuture<C, S>
where
    S: Sink<SinkItem=MessageExt, SinkError=WireError> + Send + 'static,
    C: Send + 'static,
{
    type Item = (C, S);
    type Error = WireError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

impl<C, S> ConsumingFuture<C, S>
where
    S: Sink<SinkItem=MessageExt, SinkError=WireError> + Send + 'static,
    C: Send + 'static,
{
    pub fn ok(consumer: C, sink: S) -> Self {
        use tokio::prelude::future::IntoFuture;

        ConsumingFuture(Box::new(Ok((consumer, sink)).into_future()))
    }

    pub fn from_send(consumer: C, send: sink::Send<S>) -> Self {
        ConsumingFuture(Box::new(send.map(|s| (consumer, s))))
    }

    pub fn new<F>(f: F) -> Self
    where
        F: Future<Item=(C, S), Error=WireError> + Send + 'static,
    {
        ConsumingFuture(Box::new(f))
    }
}

pub trait MessageConsumer {
    type Message: MessageFiltered;
    type Relevant: RelevantEvent;

    // determines which method will call
    // if TYPE is `SingleResponse` then `consume_single_response` will call and `consume` might be unimplemented
    // if TYPE is `MultipleResponse` then `consume` will call and `consume_single_response` might be unimplemented
    // TODO: uncomment it
    // const TYPE: MessageConsumerType;

    // without response or single message response
    // fn consume_single_response(self, message: Either<Self::Message, Self::Event>) -> (Self, Option<MessageExt>)
    // where
    //     Self: Sized;

    // consumes message and return future with the sink and maybe modified self
    // works synchronously
    fn consume<S>(self, sink: S, message: Either<Self::Message, Self::Relevant>) -> ConsumingFuture<Self, S>
    where
        Self: Sized + Send + 'static,
        S: Sink<SinkItem=MessageExt, SinkError=WireError> + Send + 'static;
}

pub trait MessageConsumerChain {
    fn process<S>(self, sink: S, message: Either<MessageExt, Event>) -> ConsumingFuture<Self, S>
    where
        Self: Sized + Send + 'static,
        S: Sink<SinkItem=MessageExt, SinkError=WireError> + Send + 'static;
}

impl MessageConsumerChain for () {
    fn process<S>(self, sink: S, message: Either<MessageExt, Event>) -> ConsumingFuture<Self, S>
    where
        Self: Sized + Send,
        S: Sink<SinkItem=MessageExt, SinkError=WireError> + Send + 'static,
    {
        println!("WARNING: skipped message {:?}", message);
        ConsumingFuture::ok(self, sink)
    }
}

impl<X, XS> MessageConsumerChain for (X, XS)
where
    X: MessageConsumer + Send + 'static,
    XS: MessageConsumerChain + Send + 'static,
{
    fn process<S>(self, s: S, message: Either<MessageExt, Event>) -> ConsumingFuture<Self, S>
    where
        Self: Sized + Send + 'static,
        S: Sink<SinkItem=MessageExt, SinkError=WireError> + Send + 'static,
    {
        let m = match message {
            Either::Left(m) => <X::Message as MessageFiltered>::filter(m).map(Either::Left).map_err(Either::Left),
            Either::Right(c) => <X::Relevant as RelevantEvent>::filter(c).map(Either::Right).map_err(Either::Right),
        };

        let (x, xs) = self;
        match m {
            Ok(m) => ConsumingFuture::new(x.consume(s, m).map(|(x, s)| ((x, xs), s))),
            Err(m) => ConsumingFuture::new(xs.process(s, m).map(|(xs, s)| ((x, xs), s))),
        }
    }
}
