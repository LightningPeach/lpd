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

pub trait MessageConsumer {
    type Message: MessageFiltered;

    // consumes message and return future with the sink and maybe modified self
    // works synchronously
    // TODO: get rid of static
    fn consume<S>(self, sink: S, message: Self::Message) -> Box<dyn Future<Item=(Self, S), Error=WireError>>
    where
        Self: Sized,
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static;

    fn try<S>(self, sink: S, message: Message) -> Result<Box<dyn Future<Item=(Self, S), Error=WireError>>, (Self, S, Message)>
    where
        Self: Sized + 'static,
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        match Self::Message::filter(message) {
            Ok(message) => Ok(Box::new(self.consume(sink, message))),
            Err(message) => Err((self, sink, message)),
        }
    }
}

pub trait MessageConsumerChain {
    fn process<S>(self, sink: S, message: Message) -> Result<Box<dyn Future<Item=(Self, S), Error=WireError>>, (Self, S, Message)>
    where
        Self: Sized,
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static;
}

impl MessageConsumerChain for () {
    fn process<S>(self, sink: S, message: Message) -> Result<Box<dyn Future<Item=(Self, S), Error=WireError>>, (Self, S, Message)>
    where
        Self: Sized,
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        Err((self, sink, message))
    }
}

impl<X, XS> MessageConsumerChain for (X, XS)
where
    X: MessageConsumer + 'static,
    XS: MessageConsumerChain + 'static,
{
    fn process<S>(self, sink: S, message: Message) -> Result<Box<dyn Future<Item=(Self, S), Error=WireError>>, (Self, S, Message)>
    where
        Self: Sized,
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        let (x, xs) = self;
        match x.try(sink, message) {
            Ok(f) => Ok(Box::new(f.map(|(x, s)| ((x, xs), s)))),
            Err((x, s, m)) => match xs.process(s, m) {
                Ok(f) => Ok(Box::new(f.map(|(xs, s)| ((x, xs), s)))),
                Err((xs, s, m)) => Err(((x, xs), s, m)),
            },
        }
    }
}
