use binformat::WireError;
use super::Message;

use tokio::prelude::Future;
use tokio::prelude::Sink;

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

pub trait MessageConsumer {
    type Message: MessageFiltered;

    // consumes message and return future with the sink and maybe modified self
    // works synchronously
    fn consume<S>(self, sink: S, message: Self::Message) -> Box<dyn Future<Item=(Self, S), Error=WireError> + Send + 'static>
    where
        Self: Sized,
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static;

    fn filter<S>(self, sink: S, message: Message) -> Result<Box<dyn Future<Item=(Self, S), Error=WireError> + Send + 'static>, (Self, S, Message)>
    where
        Self: Sized + 'static,
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        match Self::Message::filter(message) {
            Ok(message) => Ok(self.consume(sink, message)),
            Err(message) => Err((self, sink, message)),
        }
    }
}

pub trait MessageConsumerChain {
    fn process<S>(self, sink: S, message: Message) -> Result<Box<dyn Future<Item=(Self, S), Error=WireError> + Send + 'static>, (Self, S, Message)>
    where
        Self: Sized,
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static;
}

impl MessageConsumerChain for () {
    fn process<S>(self, sink: S, message: Message) -> Result<Box<dyn Future<Item=(Self, S), Error=WireError> + Send + 'static>, (Self, S, Message)>
    where
        Self: Sized,
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        use tokio::prelude::future::IntoFuture;

        println!("WARNING: skipped message {:?}", message);
        // always Ok, so could unwrap
        Ok(Box::new(Ok((self, sink)).into_future()))
    }
}

impl<X, XS> MessageConsumerChain for (X, XS)
where
    X: MessageConsumer + Send + 'static,
    XS: MessageConsumerChain + Send + 'static,
{
    fn process<S>(self, sink: S, message: Message) -> Result<Box<dyn Future<Item=(Self, S), Error=WireError> + Send + 'static>, (Self, S, Message)>
    where
        Self: Sized,
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        let (x, xs) = self;
        match x.filter(sink, message) {
            Ok(f) => Ok(Box::new(f.map(|(x, s)| ((x, xs), s)))),
            Err((x, s, m)) => match xs.process(s, m) {
                Ok(f) => Ok(Box::new(f.map(|(xs, s)| ((x, xs), s)))),
                Err((xs, s, m)) => Err(((x, xs), s, m)),
            },
        }
    }
}
