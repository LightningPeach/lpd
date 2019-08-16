use dependencies::secp256k1;
use dependencies::tokio;

use tokio::io;
use tokio::codec::{Decoder, Framed};
use tokio::prelude::Future;

use secp256k1::{PublicKey, SecretKey};
use std::time::Duration;

use super::handshake::{Machine, HandshakeIn, HandshakeOut, HandshakeError};

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
        // TODO(mkl): make this as configure parameter
        Duration::new(5, 0)
    }

    pub fn outgoing(
        stream: T,
        local_secret: SecretKey,
        remote_public: PublicKey,
    ) -> impl Future<Item = Self, Error = HandshakeError> {
        use tokio::prelude::IntoFuture;
        dbg!("BrontideStream::outgoing()");
        HandshakeOut::new(local_secret, remote_public)
            .map_err(|err| {
                HandshakeError::Crypto(err, "Cannot create handshake".to_owned())
            })
            .and_then(|noise| noise.gen_act_one())
            .into_future()
            .and_then(|(a, noise)| {
                io::write_all(stream, a)
                    .map_err(|err| {
                        HandshakeError::Io(err, "Cannot write ActOne".to_owned())
                    })
                    .map(|(stream, _)| (noise, stream))
            }).and_then(|(noise, stream)| {
                io::read_exact(stream, Default::default())
                    .map_err(|err| {
                        HandshakeError::Io(err, "Cannot read ActTwo".to_owned())
                    })
                    .and_then(|(stream, a)| {
                        let noise = noise.receive_act_two(a)?;
                        Ok((stream, noise.gen_act_three()?))
                    })
            }).and_then(|(stream, (a, noise))| {
                io::write_all(stream, a)
                    .map_err(|err| {
                        HandshakeError::Io(err, "Cannot write ActThree".to_owned())
                    })
                    .map(|(stream, _)| BrontideStream {
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
        println!("BrontideStream::incoming(stream={:p})", &stream);
        io::read_exact(stream, Default::default())
            .timeout(Self::read_timeout())
            // TODO(mkl): is it actually correct to assume that it is always timeout error
            .map_err(|err| {
                HandshakeError::IoTimeout(err, "Timeout reading ActOne".to_owned())
            })
            .and_then(move |(stream, a)| {
                HandshakeIn::new(local_secret)
                    .map_err(|err| {
                        HandshakeError::Crypto(err, "Error creating new HandshakeIn".to_owned())
                    })
                    .and_then(|noise| {
                        let noise = noise.receive_act_one(a)?;
                        Ok((stream, noise.gen_act_two()?))
                    })
            }).and_then(|(stream, (a, noise))| {
                io::write_all(stream, a)
                    .map_err(|err| {
                        HandshakeError::Io(err, "error writing ActTwo".to_owned())
                    })
                    .map(|(stream, _)| (noise, stream))
            }).and_then(|(noise, stream)| {
                io::read_exact(stream, Default::default())
                    .timeout(Self::read_timeout())
                    // TODO(mkl): is it actually correct to assume that it is always timeout error
                    .map_err(|err| {
                        HandshakeError::IoTimeout(err, "error reading ActThree".to_owned())
                    })
                    .and_then(|(stream, a)| {
                        Ok(BrontideStream {
                            noise: noise.receive_act_three(a)?,
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
