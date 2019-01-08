use std::sync::{Arc, RwLock};

use secp256k1::{SecretKey, PublicKey};
use tokio::prelude::{Future, AsyncRead, AsyncWrite, Sink};
use tokio::executor::Spawn;
use futures::sync::mpsc::Receiver;
use wire::{Message, Signature, SignError, MessageConsumer};
use binformat::WireError;

use super::address::{AbstractAddress, ConnectionStream, Command};

use state::DB;
use brontide::BrontideStream;

use routing::{State, SharedState};

use std::path::Path;
use either::Either;

#[cfg(feature = "rpc")]
use interface::routing::{LightningNode, ChannelEdge};

pub struct Node {
    peers: Vec<PublicKey>,
    shared_state: SharedState,
    db: Arc<RwLock<DB>>,
    secret: SecretKey,
}

pub struct Remote {
    db: Arc<RwLock<DB>>,
    public: PublicKey,
}

impl MessageConsumer for Remote {
    type Message = Message;

    fn consume<S>(self, sink: S, message: Self::Message) -> Box<dyn Future<Item=(Self, S), Error=WireError> + Send + 'static>
    where
        Self: Sized,
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        use tokio::prelude::future::IntoFuture;

        // TODO: process the message using db and public
        let _ = (&self.db, &self.public);
        let _ = message;

        Box::new(Ok((self, sink)).into_future())
    }
}

impl Node {
    pub fn new<P: AsRef<Path>>(secret: [u8; 32], path: P) -> Self {
        use state::DBBuilder;

        let db = DBBuilder::default().user::<State>().build(path).unwrap();
        let p_db = Arc::new(RwLock::new(db));

        Node {
            peers: Vec::new(),
            shared_state: SharedState(Arc::new(RwLock::new(State::new(p_db.clone())))),
            db: p_db,
            secret: SecretKey::from_slice(&secret[..]).unwrap(),
        }
    }

    fn add(&mut self, remote_public: PublicKey) -> Either<PublicKey, Remote> {
        if self.peers.contains(&remote_public) {
            Either::Left(remote_public)
        } else {
            self.peers.push(remote_public.clone());
            Either::Right(Remote {
                db: self.db.clone(),
                public: remote_public,
            })
        }
    }

    fn process_connection<S>(p_self: Arc<RwLock<Self>>, stream: BrontideStream<S>) -> Spawn
    where
        S: AsyncRead + AsyncWrite + Send + 'static,
    {
        use tokio::prelude::stream::Stream;
        use wire::MessageConsumerChain;

        let remote_public = stream.remote_key().clone();
        let (sink, stream) = stream.framed().split();

        let p_graph = p_self.read().unwrap().shared_state.clone();
        match p_self.write().unwrap().add(remote_public) {
            Either::Left(k) => {
                use futures::future::ok;
                println!("WARNING: {} is connected, ignoring", k);
                tokio::spawn(ok(()))
            },
            Either::Right(peer) => {
                println!("INFO: new peer {}", peer.public);

                let processor = (p_graph, (peer, ()));
                let connection = stream
                    .fold((processor, sink), |(processor, sink), message| {
                        processor.process(sink, message).ok().unwrap()
                    })
                    .map_err(|e| panic!("{:?}", e))
                    .map(|_| ());

                tokio::spawn(connection)
            }
        }
    }

    pub fn listen<A>(p_self: Arc<RwLock<Self>>, address: &A, control: Receiver<Command<A>>) -> Result<(), A::Error>
    where
        A: AbstractAddress + Send + 'static,
    {
        use tokio::prelude::stream::Stream;
        use secp256k1::Secp256k1;

        let secret = p_self.read().unwrap().secret.clone();
        let pk = PublicKey::from_secret_key(&Secp256k1::new(), &secret);
        println!("INFO: pk {:?}", hex::encode(&pk.serialize()[..]));
        let server = ConnectionStream::new(address, control, secret)?
            .map_err(|e| println!("{:?}", e))
            .for_each(move |stream| Self::process_connection(p_self.clone(), stream));
        tokio::run(server);
        Ok(())
    }

    pub fn sign_message(p_self: Arc<RwLock<Self>>, message: Vec<u8>) -> Result<Signature, SignError> {
        use wire::{Signed, SignedData};
        use binformat::SerdeRawVec;

        let secret_key = From::from(p_self.read().unwrap().secret.clone());
        Signed::sign(SignedData(SerdeRawVec(message)), &secret_key).map(|s| s.signature)
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
    pub fn list_peers(p_self: Arc<RwLock<Self>>) -> Vec<PublicKey> {
        p_self.read().unwrap().peers.clone()
    }

    #[cfg(feature = "rpc")]
    pub fn describe_graph(p_self: Arc<RwLock<Self>>, include_unannounced: bool) -> (Vec<ChannelEdge>, Vec<LightningNode>) {
        p_self.read().unwrap().shared_state.clone().0.read().unwrap().describe(include_unannounced)
    }
}
