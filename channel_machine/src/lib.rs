#![forbid(unsafe_code)]

mod b_box;
pub use self::b_box::{ChannelState, InitialState, ReadyState, OpeningState, WaitFundingCreatedData, WaitFundingLockedData};

use processor::{MessageConsumer, ConsumingFuture};
use wire::Message;
use binformat::WireError;

use tokio::prelude::Sink;
use either::Either;

impl MessageConsumer for ChannelState {
    type Message = Message;
    type Relevant = ();

    fn consume<S>(self, sink: S, message: Either<Self::Message, Self::Relevant>) -> ConsumingFuture<Self, S>
    where
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        match message {
            Either::Left(message) => {
                match self.next(message) {
                    (state, Some(response)) => {
                        let send = sink.send(response);
                        ConsumingFuture::from_send(state, send)
                    },
                    (state, None) => ConsumingFuture::ok(state, sink),
                }
            },
            Either::Right(event) => {
                match event {
                    // process events here
                    () => ConsumingFuture::ok(self, sink)
                }
            },
        }
    }
}
