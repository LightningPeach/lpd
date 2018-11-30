#![forbid(unsafe_code)]

extern crate grpc;
extern crate interface;
extern crate routing;
extern crate wire;
#[macro_use]
extern crate lazy_static;

mod channel_impl;
mod routing_impl;
mod payment_impl;

pub use self::channel_impl::service as channel_service;
pub use self::routing_impl::service as routing_service;
pub use self::payment_impl::service as payment_service;

use routing::State;

lazy_static! {
    static ref STATE: State = {
        let mut state = State::new("db").unwrap();
        state.load();
        state
    };
}
