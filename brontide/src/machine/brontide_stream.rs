use tokio::io;
use tokio::codec::{Decoder, Framed};
use tokio::prelude::Future;

use secp256k1::{PublicKey, SecretKey};
use std::time::Duration;

use super::handshake::{Machine, HandshakeNew, HandshakeInitiator, HandshakeError};

pub struct BrontideStream<T>
where
    T: io::AsyncRead + io::AsyncWrite,
{
    noise: Machine,
    stream: T,
}

impl<T> BrontideStream<T>
where
    T: io::AsyncRead + io::AsyncWrite,
{
    // HANDSHAKE_READ_TIMEOUT is a read timeout that will be enforced when
    // waiting for data payloads during the various acts of Brontide. If
    // the remote party fails to deliver the proper payload within this
    // time frame, then we'll fail the connection.
    fn read_timeout() -> Duration {
        Duration::new(5, 0)
    }

    pub fn outgoing(
        stream: T,
        local_secret: SecretKey,
        remote_public: PublicKey,
    ) -> impl Future<Item = Self, Error = HandshakeError> {
        use tokio::prelude::IntoFuture;

        HandshakeInitiator::new(local_secret, remote_public)
            .map_err(HandshakeError::Crypto)
            .and_then(|noise| noise.gen_act_one())
            .into_future()
            .and_then(move |(a, noise)| {
                io::write_all(stream, a)
                    .map_err(HandshakeError::Io)
                    .map(move |(stream, _)| (noise, stream))
            }).and_then(|(noise, stream)| {
                io::read_exact(stream, Default::default())
                    .map_err(HandshakeError::Io)
                    .and_then(move |(stream, a)| {
                        let noise = noise.recv_act_two(a)?;
                        Ok((stream, noise.gen_act_three()?))
                    })
            }).and_then(|(stream, (a, noise))| {
                io::write_all(stream, a)
                    .map_err(HandshakeError::Io)
                    .map(move |(stream, _)| BrontideStream {
                        noise: noise,
                        stream: stream,
                    })
            })
    }

    pub fn incoming(
        stream: T,
        local_secret: SecretKey,
    ) -> impl Future<Item = Self, Error = HandshakeError> {
        use tokio::prelude::FutureExt;

        io::read_exact(stream, Default::default())
            .timeout(Self::read_timeout())
            .map_err(HandshakeError::IoTimeout)
            .and_then(move |(stream, a)| {
                HandshakeNew::new(local_secret)
                    .map_err(HandshakeError::Crypto)
                    .and_then(move |noise| {
                        let noise = noise.recv_act_one(a)?;
                        Ok((stream, noise.gen_act_two()?))
                    })
            }).and_then(|(stream, (a, noise))| {
                io::write_all(stream, a)
                    .map_err(HandshakeError::Io)
                    .map(move |(stream, _)| (noise, stream))
            }).and_then(|(noise, stream)| {
                io::read_exact(stream, Default::default())
                    .timeout(Self::read_timeout())
                    .map_err(HandshakeError::IoTimeout)
                    .and_then(move |(stream, a)| {
                        Ok(BrontideStream {
                            noise: noise.recv_act_three(a)?,
                            stream: stream,
                        })
                    })
            })
    }

    pub fn remote_key(&self) -> PublicKey {
        self.noise.remote_static()
    }

    pub fn framed(self) -> Framed<T, Machine> {
        self.noise.framed(self.stream)
    }
}

impl<T> AsRef<T> for BrontideStream<T>
where
    T: io::AsyncRead + io::AsyncWrite,
{
    fn as_ref(&self) -> &T {
        &self.stream
    }
}

impl<T> AsMut<T> for BrontideStream<T>
where
    T: io::AsyncRead + io::AsyncWrite,
{
    fn as_mut(&mut self) -> &mut T {
        &mut self.stream
    }
}
