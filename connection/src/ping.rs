use dependencies::tokio;
use dependencies::either;
use dependencies::chrono;

use wire::{Message, MessageExt, Ping, Pong};
use processor::{MessageConsumer, MessageFiltered, RelevantEvent, ConsumingFuture};
use internal_event::Event;
use binformat::WireError;
use tokio::prelude::Sink;
use either::Either;

#[derive(Default, Debug)]
pub struct PingContext {
    timestamp: i64,
    tick: u8,
}

#[derive(Debug)]
pub struct PingMessage(Ping);

impl MessageFiltered for PingMessage {
    fn filter(v: MessageExt) -> Result<Self, MessageExt> {
        match v.message {
            Message::Ping(ping) => Ok(PingMessage(ping)),
            _ => Err(v),
        }
    }
}

#[derive(Debug)]
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
        S: Sink<SinkItem=MessageExt, SinkError=WireError> + Send + 'static,
    {
        use chrono::prelude::*;

        dbg!(&message);

        let mut this = self;
        match message {
            Either::Left(PingMessage(ping)) => {
                this.timestamp = Utc::now().timestamp();
                let pong = Message::Pong(Pong::new(&ping));
                ConsumingFuture::from_send(this, sink.send(pong.into()))
            },
            Either::Right(PingEvent) => {
                this.tick += 1;
                if this.tick == 30 || Utc::now().timestamp() - this.timestamp >= 30 {
                    this.tick = 0;
                    let ping = Message::Ping(Ping::new(256, 256).unwrap());
                    ConsumingFuture::from_send(this, sink.send(ping.into()))
                } else {
                    ConsumingFuture::ok(this, sink)
                }
            }
        }
    }
}
