use wire::{Message, Ping, Pong};
use processor::{MessageConsumer, MessageFiltered, RelevantEvent, Event, ConsumingFuture};
use binformat::WireError;
use tokio::prelude::Sink;
use either::Either;

#[derive(Default)]
pub struct PingContext {
    timestamp: i64,
    tick: u8,
}

pub struct PingMessage(Ping);

impl MessageFiltered for PingMessage {
    fn filter(v: Message) -> Result<Self, Message> {
        match v {
            Message::Ping(ping) => Ok(PingMessage(ping)),
            _ => Err(v),
        }
    }
}

pub struct PingEvent;

impl RelevantEvent for PingEvent {
    fn filter(v: Event) -> Result<Self, Event> {
        match v {
            Event::TimerTick => Ok(PingEvent),
            v @ _ => Err(v),
        }
    }
}

impl MessageConsumer for PingContext {
    type Message = PingMessage;
    type Relevant = PingEvent;

    fn consume<S>(self, sink: S, message: Either<Self::Message, Self::Relevant>) -> ConsumingFuture<Self, S>
    where
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        use chrono::prelude::*;

        let mut this = self;
        match message {
            Either::Left(PingMessage(ping)) => {
                this.timestamp = Utc::now().timestamp();
                let pong = Message::Pong(Pong::new(&ping));
                ConsumingFuture::from_send(this, sink.send(pong))
            },
            Either::Right(PingEvent) => {
                this.tick += 1;
                if this.tick == 30 || Utc::now().timestamp() - this.timestamp >= 30 {
                    this.tick = 0;
                    let ping = Message::Ping(Ping::new(256, 256).unwrap());
                    ConsumingFuture::from_send(this, sink.send(ping))
                } else {
                    ConsumingFuture::ok(this, sink)
                }
            }
        }
    }
}
