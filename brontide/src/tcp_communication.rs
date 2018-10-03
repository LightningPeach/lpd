use secp256k1::{PublicKey, SecretKey};
use std::{net, io};

use super::machine::{HandshakeError, Machine};

// NetAddress represents information pertaining to the identity and network
// reachability of a peer. Information stored includes the node's identity
// public key for establishing a confidential+authenticated connection, the
// service bits it supports, and a TCP address the node is reachable at.
#[derive(Debug)]
pub struct NetAddress {
    // identity_key is the long-term static public key for a node. This node is
    // used throughout the network as a node's identity key. It is used to
    // authenticate any data sent to the network on behalf of the node, and
    // additionally to establish a confidential+authenticated connection with
    // the node.
    pub identity_key: PublicKey,

    // address is the IP address and port of the node. This is left
    // general so that multiple implementations can be used.
    pub socket: net::SocketAddr,
}

pub struct Stream {
    noise: Machine,
    raw: net::TcpStream,
}

impl Stream {
    pub fn connect(secret_key: SecretKey, address: NetAddress) -> Result<Self, HandshakeError> {
        let mut stream = net::TcpStream::connect(address.socket).map_err(HandshakeError::Io)?;
        let mut machine = Machine::new::<fn(&mut Machine)>(true, secret_key, address.identity_key, &[])
            .map_err(HandshakeError::Crypto)?;

        // async
        let _ = machine.handshake(&mut stream)?;

        Ok(Stream {
            noise: machine,
            raw: stream,
        })
    }

    pub fn accept(listener: &net::TcpListener, secret_key: SecretKey) -> Result<(Self, NetAddress), HandshakeError> {
        let (mut stream, address) = listener.accept().map_err(HandshakeError::Io)?;
        let mut machine = Machine::new::<fn(&mut Machine)>(false, secret_key, PublicKey::new(), &[])
            .map_err(HandshakeError::Crypto)?;

        let public_key = machine.handshake(&mut stream)?;

        Ok((Stream {
            noise: machine,
            raw: stream,
        }, NetAddress {
            identity_key: public_key,
            socket: address,
        }))
    }
}

// TODO: impl io::BufRead
impl io::Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.noise.read_message(&mut self.raw).map(|v| {
            buf.copy_from_slice(v.as_slice());
            v.len()
        })
    }
}

impl io::Write for Stream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.noise.write_message(&mut self.raw, buf).map(|()| buf.len())
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        io::Write::flush(&mut self.raw)
    }
}
