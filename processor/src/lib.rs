#![forbid(unsafe_code)]

use binformat::WireError;
use wire::Message;

use tokio::prelude::Future;
use tokio::prelude::Sink;
use either::Either;

#[derive(Debug)]
pub enum Event {
    DirectCommand(DirectCommand),
    TimerTick,
}

#[derive(Debug)]
pub enum DirectCommand {
    _Nothing,
}

pub trait MessageFiltered
    where
        Self: Sized,
{
    fn filter(v: Message) -> Result<Self, Message>;
}

impl MessageFiltered for Message {
    fn filter(v: Message) -> Result<Self, Message> {
        Ok(v)
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

pub type ConsumingFuture<C, S> = Box<dyn Future<Item=(C, S), Error=WireError> + Send + 'static>;

pub trait MessageConsumer {
    type Message: MessageFiltered;
    type Relevant: RelevantEvent;

    // determines which method will call
    // if TYPE is `SingleResponse` then `consume_single_response` will call and `consume` might be unimplemented
    // if TYPE is `MultipleResponse` then `consume` will call and `consume_single_response` might be unimplemented
    // TODO: uncomment it
    // const TYPE: MessageConsumerType;

    // without response or single message response
    // fn consume_single_response(self, message: Either<Self::Message, Self::Event>) -> (Self, Option<Message>)
    // where
    //     Self: Sized;

    // consumes message and return future with the sink and maybe modified self
    // works synchronously
    fn consume<S>(self, sink: S, message: Either<Self::Message, Self::Relevant>) -> ConsumingFuture<Self, S>
        where
            Self: Sized,
            S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static;
}

pub trait MessageConsumerChain {
    fn process<S>(self, sink: S, message: Either<Message, Event>) -> ConsumingFuture<Self, S>
        where
            Self: Sized,
            S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static;
}

impl MessageConsumerChain for () {
    fn process<S>(self, sink: S, message: Either<Message, Event>) -> ConsumingFuture<Self, S>
        where
            Self: Sized,
            S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        use tokio::prelude::future::IntoFuture;

        println!("WARNING: skipped message {:?}", message);
        Box::new(Ok((self, sink)).into_future())
    }
}

impl<X, XS> MessageConsumerChain for (X, XS)
    where
        X: MessageConsumer + Send + 'static,
        XS: MessageConsumerChain + Send + 'static,
{
    fn process<S>(self, s: S, message: Either<Message, Event>) -> ConsumingFuture<Self, S>
        where
            Self: Sized,
            S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        let m = match message {
            Either::Left(m) => <X::Message as MessageFiltered>::filter(m).map(Either::Left).map_err(Either::Left),
            Either::Right(c) => <X::Relevant as RelevantEvent>::filter(c).map(Either::Right).map_err(Either::Right),
        };

        let (x, xs) = self;
        match m {
            Ok(m) => Box::new(x.consume(s, m).map(|(x, s)| ((x, xs), s))),
            Err(m) => Box::new(xs.process(s, m).map(|(xs, s)| ((x, xs), s))),
        }
    }
}
