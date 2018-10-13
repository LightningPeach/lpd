use tokio::io;
use tokio::codec::{Decoder, Framed};
use tokio::prelude::Future;

use secp256k1::{PublicKey, SecretKey};
use std::time::Duration;

use super::{Machine, HandshakeError};

pub struct BrontideStream<T> where T: io::AsyncRead + io::AsyncWrite {
    // the Machine holds a lot of byte arrays,
    // combined with tokio runtime it overflows stack,
    // let us put it in the box
    noise: Box<Machine>,
    stream: T,
}

impl<T> BrontideStream<T> where T: io::AsyncRead + io::AsyncWrite {
    // HANDSHAKE_READ_TIMEOUT is a read timeout that will be enforced when
    // waiting for data payloads during the various acts of Brontide. If
    // the remote party fails to deliver the proper payload within this
    // time frame, then we'll fail the connection.
    fn read_timeout() -> Duration {
        Duration::new(5, 0)
    }

    pub fn outgoing(stream: T, local_secret: SecretKey, remote_public: PublicKey) -> impl Future<Item=Self, Error=HandshakeError> {
        use tokio::prelude::{FutureExt, IntoFuture};

        Machine::new::<fn(&mut Machine)>(true, local_secret, remote_public, &[])
            .map_err(HandshakeError::Crypto)
            .and_then(|mut noise|
                noise.gen_act_one()
                    .map(|a| (Box::new(noise), a))
            )
            .into_future()
            .and_then(move |(noise, a)|
                io::write_all(stream, a)
                    .map_err(HandshakeError::Io)
                    .map(move |(stream, _)| BrontideStream { noise: noise, stream: stream })
            )
            .and_then(|BrontideStream { noise: mut noise, stream: stream }|
                io::read_exact(stream, Default::default())
                    .map_err(HandshakeError::Io)
                    .and_then(move |(stream, a)| {
                        noise.recv_act_two(a)?;
                        let a = noise.gen_act_three()?;
                        Ok((BrontideStream { noise: noise, stream: stream }, a))
                    })
            )
            .and_then(|(BrontideStream { noise: noise, stream: stream }, a)|
                io::write_all(stream, a)
                    .map_err(HandshakeError::Io)
                    .map(move |(stream, _)| BrontideStream { noise: noise, stream: stream })
            )
    }

    pub fn incoming(stream: T, local_secret: SecretKey) -> impl Future<Item=(Self, PublicKey), Error=HandshakeError> {
        use tokio::prelude::FutureExt;

        io::read_exact(stream, Default::default())
            .timeout(Self::read_timeout())
            .map_err(HandshakeError::IoTimeout)
            .and_then(move |(stream, a)| {
                Machine::new::<fn(&mut Machine)>(false, local_secret, PublicKey::new(), &[])
                    .map_err(HandshakeError::Crypto)
                    .map(Box::new)
                    .and_then(move |mut noise| {
                        noise.recv_act_one(a)?;
                        let a = noise.gen_act_two()?;
                        Ok((BrontideStream { noise: noise, stream: stream }, a))
                    })
            })
            .and_then(|(BrontideStream { noise: noise, stream: stream }, a)|
                io::write_all(stream, a)
                    .map_err(HandshakeError::Io)
                    .map(move |(stream, _)| BrontideStream { noise: noise, stream: stream })
            )
            .and_then(|BrontideStream { noise: mut noise, stream: stream }|
                io::read_exact(stream, Default::default())
                    .timeout(Self::read_timeout())
                    .map_err(HandshakeError::IoTimeout)
                    .and_then(move |(stream, a)| {
                        noise.recv_act_three(a)?;
                        Ok(BrontideStream { noise: noise, stream: stream })
                    })
            )
            .map(|this| {
                let k = this.noise.handshake_state.remote_static.clone();
                (this, k)
            })
    }

    pub fn framed(self) -> Framed<T, Box<Machine>> {
        self.noise.framed(self.stream)
    }
}

impl<T> AsRef<T> for BrontideStream<T> where T: io::AsyncRead + io::AsyncWrite {
    fn as_ref(&self) -> &T {
        &self.stream
    }
}

impl<T> AsMut<T> for BrontideStream<T> where T: io::AsyncRead + io::AsyncWrite {
    fn as_mut(&mut self) -> &mut T {
        &mut self.stream
    }
}
