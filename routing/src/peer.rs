use wire::Message;
use wire::Address;
use wire::PublicKey;

use brontide::MachineRead;
use brontide::MachineWrite;
use brontide::MessageConsumer;
use brontide::MessageSource;
use brontide::BrontideError;
use brontide::tcp_communication::Stream;
use brontide::tcp_communication::Listener;

use rand;

pub trait Peer {
    fn send(&mut self, message: Message) -> Result<(), BrontideError>;
    fn receive(&mut self) -> Result<Message, BrontideError>;
}

pub struct TcpSelf {
    private_key: [u8; 32],
    peers: Vec<TcpPeer>,
}

pub struct TcpPeer {
    public_key: PublicKey,
    stream: Stream,
}

impl TcpSelf {
    pub fn new() -> Self {
        TcpSelf {
            private_key: rand::random(),
            peers: Vec::new(),
        }
    }

    pub fn connect_peer(&mut self, public_key: PublicKey, address: Address) -> Result<(), ()> {
        use brontide::tcp_communication::NetAddress;
        use secp256k1::Secp256k1;
        use secp256k1::SecretKey;

        let socket_address = address
            .into_socket_address()
            .map_err(|_| ())?;

        let net_address = NetAddress {
            identity_key: public_key.into(),
            address: socket_address,
        };

        let key = SecretKey::from_slice(&Secp256k1::new(), &self.private_key[0..])
            .map_err(|_| ())?;

        let peer = Stream::dial(key, net_address)
            .map(|stream| TcpPeer { public_key: public_key, stream: stream, })
            .map_err(|_| ())?;

        self.peers.push(peer);

        Ok(())
    }
}

impl Peer for TcpPeer {
    fn send(&mut self, message: Message) -> Result<(), BrontideError> {
        self.stream.as_write().send(message)
    }

    fn receive(&mut self) -> Result<Message, BrontideError> {
        self.stream.as_read().receive()
    }
}
