use std::sync::{Arc, RwLock};

use secp256k1::{SecretKey, PublicKey};
use tokio::prelude::{Future, AsyncRead, AsyncWrite};
use tokio::executor::Spawn;
use futures::sync::mpsc::Receiver;
use wire::{Message, Signature, SignError};

use super::address::{AbstractAddress, ConnectionStream, Command};

use state::{DB, DBError};
use brontide::BrontideStream;

use std::path::Path;

pub fn database<P: AsRef<Path>>(path: P) -> Result<DB, DBError> {
    use state::DBBuilder;

    DBBuilder::default()
        // add here database inhabitants
        //.register::<SomeType>()
        .build(path)
}

pub struct Node {
    peers: Vec<PublicKey>,
    db: DB,
    secret: SecretKey,
}

pub struct Remote {
    public: PublicKey,
}

impl Remote {
    pub fn process_message(&mut self, db: &mut DB, message: Message) {
        let _ = (message, &mut self.public, db);
    }
}

impl Node {
    pub fn new<P: AsRef<Path>>(secret: [u8; 32], path: P) -> Self {
        Node {
            peers: Vec::new(),
            db: database(path).unwrap(),
            secret: SecretKey::from_slice(&secret[..]).unwrap(),
        }
    }

    fn add(&mut self, remote_public: PublicKey) -> Option<PublicKey> {
        if self.peers.contains(&remote_public) {
            Some(remote_public)
        } else {
            self.peers.push(remote_public);
            None
        }
    }

    fn process_connection<S>(p_self: Arc<RwLock<Self>>, stream: BrontideStream<S>) -> Spawn
    where
        S: AsyncRead + AsyncWrite + Send + 'static,
    {
        use tokio::prelude::stream::Stream;

        let remote_public = stream.remote_key().clone();
        let (sink, stream) = stream.framed().split();

        // nll will fix it,
        // p_self is borrowed, but it is not used in match's `None` arm
        // however, the compiler still complains, so let's clone the pointer here
        let shared = p_self.clone();
        match p_self.write().unwrap().add(remote_public) {
            Some(k) => {
                use futures::future::ok;
                println!("WARNING: {} is connected, ignoring", k);
                tokio::spawn(ok(()))
            },
            None => {
                println!("INFO: new peer {:?}", hex::encode(&remote_public.serialize()[..]));

                let peer = Remote {
                    public: remote_public,
                };

                let connection = stream
                    .map_err(|e| println!("{:?}", e))
                    .fold(peer, move |mut peer, message| {
                        println!("{:?}", message);
                        // process message using sink
                        let _ = &sink;
                        peer.process_message(&mut shared.clone().write().unwrap().db, message);
                        Ok(peer)
                    })
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
        use secp256k1::Secp256k1;;

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
    pub fn list_peers(p_self: Arc<RwLock<Self>>) -> Vec<PublicKey> {
        p_self.read().unwrap().peers.clone()
    }
}
