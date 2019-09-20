use std::sync::{Arc, RwLock, Mutex};

use dependencies::secp256k1;
use dependencies::tokio;
use dependencies::futures;
use dependencies::either;

use wallet_lib::interface::Wallet;

use secp256k1::{SecretKey, PublicKey};
use tokio::prelude::{Future, AsyncRead, AsyncWrite, Sink};
use tokio::executor::Spawn;
use futures::sync::mpsc;
use secp256k1::Signature;
use wire::{Message, MessageExt};
use processor::{MessageConsumer, ConsumingFuture};
use binformat::WireError;

use crate::address::TransportError;
use super::address::{AbstractAddress, ConnectionStream, Command, Connection};
use super::ping::PingContext;
use super::blockchain::Blockchain;

use state::DB;

use routing::{State, SharedState};
use channel_machine::ChannelState;

use std::path::Path;
use either::Either;

#[cfg(feature = "rpc")]
use interface::routing::{LightningNode, ChannelEdge, Info};
use std::collections::HashMap;

pub struct Node {
    peers: Vec<PublicKey>,
    channels: HashMap<PublicKey, mpsc::Receiver<ChannelStatus>>,
    shared_state: SharedState,
    db: Arc<RwLock<DB>>,
    secret: SecretKey,
    blockchain: Blockchain,
    wallet: Arc<Mutex<Box<dyn Wallet + Send>>>,
}

/// Represents the channel, but not the peer
pub struct Remote {
    db: Arc<RwLock<DB>>,
    wallet: Arc<Mutex<Box<dyn Wallet + Send>>>,
    public: PublicKey,
    channel: ChannelState,
    sender: mpsc::Sender<ChannelStatus>,
}

impl MessageConsumer for Remote {
    type Message = Message;
    type Relevant = ();

    fn consume<S>(mut self, sink: S, message: Either<Self::Message, Self::Relevant>) -> ConsumingFuture<Self, S>
    where
        Self: Sized,
        S: Sink<SinkItem=MessageExt, SinkError=WireError> + Send + 'static,
    {
        // TODO: use them
        let _ = (&self.db, &self.public, &self.wallet);

        println!("channel state: {:?}", self.channel);
        println!("received message: {:?}", message);

        match message {
            Either::Left(message) => {
                match self.channel.next(message) {
                    (state, Some(response)) => {
                        println!("response message: {:?}", response);
                        let send = sink.send(response.into());
                        self.channel = state;
                        ConsumingFuture::from_send(self, send)
                    },
                    (state, None) => {
                        println!("response nothing");
                        self.channel = state;
                        ConsumingFuture::ok(self, sink)
                    },
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

// TODO: move it to proper place
#[derive(Debug)]
pub enum ChannelStatus {
    Pending,
    Confirmation,
    Open,
}

impl Node {
    pub fn new<P: AsRef<Path>>(wallet: Arc<Mutex<Box<dyn Wallet + Send>>>, secret: [u8; 32], path: P) -> Self {
        use state::DBBuilder;

        let db = DBBuilder::default().user::<State>().build(path).unwrap();
        let p_db = Arc::new(RwLock::new(db));

        Node {
            peers: Vec::new(),
            channels: HashMap::new(),
            shared_state: SharedState(Arc::new(RwLock::new(State::new(p_db.clone())))),
            db: p_db,
            secret: SecretKey::from_slice(&secret[..]).unwrap(),
            blockchain: Blockchain::bitcoin(wallet.clone()),
            wallet: wallet,
        }
    }

    fn add(&mut self, remote_public: PublicKey) -> Either<PublicKey, Remote> {
        if self.peers.contains(&remote_public) {
            Either::Left(remote_public)
        } else {
            let (sender, receiver) = mpsc::channel(16);
            self.peers.push(remote_public.clone());
            self.channels.insert(remote_public.clone(), receiver);
            Either::Right(Remote {
                db: self.db.clone(),
                wallet: self.wallet.clone(),
                public: remote_public,
                channel: ChannelState::new(),
                sender: sender,
            })
        }
    }

    fn process_connection<S>(&self, peer: Remote, connection: Connection<S>) -> Spawn
    where
        S: AsyncRead + AsyncWrite + Send + 'static,
    {
        use tokio::prelude::stream::Stream;
        use processor::MessageConsumerChain;

        let (sink, stream) = connection.split();

        println!("INFO: new peer {}", peer.public);
        let peer_pubkey = peer.public.clone();

        let p_graph = self.shared_state.clone();
        let processor = (p_graph, (PingContext::default(), (peer, ())));
        // TODO(mkl)
        let connection = stream
            .fold((processor, sink), |(processor, sink), message| {
                processor.process(sink, message)
            })
            .map_err(move |err| {
                // TODO(mkl): correctly delete peer from list of connected peers
                println!("ERROR with peer: {:?}, {:?}", &peer_pubkey, err);
                ()
            })
            .map(move |_| {
                // TODO(mkl): correctly delete peer from list of connected peers
                println!("finished processing connection with peer: {:?}", &peer_pubkey);
            });

        tokio::spawn(connection)
    }

    pub fn listen<A>(p_self: Arc<RwLock<Self>>, address: &A, control: mpsc::Receiver<Command<A>>) -> Result<(), TransportError>
    where
        A: AbstractAddress + Send  + 'static,
    {
        use tokio::prelude::stream::Stream;
        use futures::future::ok;

        let secret = p_self.read().unwrap().secret.clone();
        let server = ConnectionStream::listen(address, control, secret)?
            .map_err(|e| println!("{:?}", e))
            .for_each(move |connection| {
                let remote_public = connection.remote_key();
                println!("NEW CONNECTION FROM: {:?}", remote_public);
                // TODO(mkl): rewrite this
                let maybe_peer = p_self.write().unwrap().add(remote_public);
                match maybe_peer {
                    Either::Left(pk) => {
                        println!("WARNING: {} is connected, ignoring", pk);
                        tokio::spawn(ok(()))
                    },
                    Either::Right(peer) => p_self.read().unwrap().process_connection(peer, connection),
                }
            });
        tokio::run(server);
        Ok(())
    }

    pub fn sign_message(&self, message: Vec<u8>) -> Signature {
        use common_types::{secp256k1_m::{Data, Signed}, ac};
        use secp256k1::Secp256k1;
        use binformat::SerdeRawVec;
        use wire::types::RawSignature;

        let context = Secp256k1::signing_only();
        let secret_key = From::from(self.secret.clone());
        let data = Data(SerdeRawVec(message));
        let signed: Signed<_, RawSignature> = ac::Signed::sign(data, &context, &secret_key);
        signed.signature.0
    }

    pub fn take_channel_status_stream(&mut self, remote_public: &PublicKey) -> Option<mpsc::Receiver<ChannelStatus>> {
        self.channels.remove(remote_public)
    }

    // TODO: add missing fields:
    //    pub address: ::std::string::String,
    //    pub bytes_sent: u64,
    //    pub bytes_recv: u64,
    //    pub sat_sent: ::protobuf::SingularPtrField<super::common::Satoshi>,
    //    pub sat_recv: ::protobuf::SingularPtrField<super::common::Satoshi>,
    //    pub inbound: bool,
    //    pub ping_time: i64,
    #[cfg(feature = "rpc")]
    pub fn list_peers(&self) -> Vec<PublicKey> {
        self.peers.clone()
    }

    #[cfg(feature = "rpc")]
    pub fn describe_graph(&self, include_unannounced: bool) -> (Vec<ChannelEdge>, Vec<LightningNode>) {
        self.shared_state.0.read().unwrap().describe(include_unannounced)
    }

    // TODO: add missing fields
    #[cfg(feature = "rpc")]
    pub fn get_info(&mut self) -> Info {
        use secp256k1::Secp256k1;
        use build_info::get_build_info;

        self.blockchain.sync();

        let pk = PublicKey::from_secret_key(&Secp256k1::new(), &self.secret);

        let mut info = Info::new();
        info.set_identity_pubkey(pk.to_string());
        info.set_num_peers(self.peers.len() as _);
        info.set_block_hash(self.blockchain.hash().to_string());
        info.set_block_height(self.blockchain.height());
        info.set_version(format!("{:#?}", get_build_info!()));
        info
    }

    // TODO: add missing fields
    #[cfg(feature = "rpc")]
    pub fn find_route(&self, goal: PublicKey) -> Vec<(LightningNode, ChannelEdge)> {
        use secp256k1::Secp256k1;
        let start = PublicKey::from_secret_key(&Secp256k1::new(), &self.secret);

        // goal is not included, so let's swap start and goal so starting node is not included
        self.shared_state.0.read().unwrap().path(goal.into(), start.into())
    }
}
