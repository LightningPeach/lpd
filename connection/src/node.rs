use std::sync::{Arc, RwLock, Weak};

use secp256k1::{SecretKey, PublicKey};
use tokio::prelude::{Future, Sink};
use specs::{World, System, DenseVecStorage};
use wire::Message;

use specs_derive::Component;

use super::address::AbstractAddress;

use state::{DB, DBError};
use std::path::Path;
use std::sync::mpsc::Receiver;

pub fn database<P: AsRef<Path>>(path: P) -> Result<DB, DBError> {
    use state::DBBuilder;

    DBBuilder::default()
        // add here database inhabitants
        //.register::<SomeType>()
        .build(path)
}

pub struct Node {
    world: World,
    secret: SecretKey,
    peers: Vec<PublicKey>,
}

#[derive(Component)]
pub struct Remote {
    shared: Weak<RwLock<Node>>,
    world: World,
    public: PublicKey,
}

pub struct Msg(pub Option<Message>);

impl<'a> System<'a> for Msg {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {
        let _ = data;
        let &mut Msg(ref mut maybe) = self;
        println!("message: {:?}", maybe.as_ref().unwrap());
    }
}

impl Node {
    pub fn new(secret: [u8; 32]) -> Self {
        Node {
            world: {
                let mut world = World::new();
                // world.register();
                world
            },
            secret: SecretKey::from_slice(&secret[..]).unwrap(),
            peers: Vec::new(),
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

    pub fn listen<A>(shared: Arc<RwLock<Self>>, address: &A, rx: Receiver<()>)
    where
        A: AbstractAddress,
    {
        use tokio::prelude::stream::Stream;
        use specs::RunNow;

        // TODO: poll rx to gracefully stop
        let secret = shared.read().unwrap().secret.clone();
        let server = address.listen(secret)
            .map_err(|e| println!("{:?}", e))
            .for_each(move |stream| {
                let shared = shared.clone();
                let remote_public = stream.remote_key().clone();
                let (sink, stream) = stream.framed().split();

                let mut this = shared.write().unwrap();
                match this.add(remote_public) {
                    Some(k) => {
                        use futures::future::ok;
                        println!("WARNING: {} is connected, ignoring", k);
                        tokio::spawn(ok(()))
                    },
                    None => {
                        let peer = Remote {
                            shared: Arc::downgrade(&shared),
                            world: World::new(),
                            public: remote_public,
                        };

                        let connection = stream
                            .map_err(|e| println!("{:?}", e))
                            .fold(peer, |mut peer, message| {
                                println!("{:?}", message);
                                Msg(Some(message)).run_now(&mut peer.world.res);
                                Ok(peer)
                            })
                            .map(|_| ());

                        tokio::spawn(connection)
                    }
                }
            });
        tokio::run(server);
    }

    // TODO: get rid of duplicated code
    pub fn connect<A>(shared: Arc<RwLock<Self>>, address: &A, remote_public: PublicKey)
    where
        A: AbstractAddress,
    {
        use tokio::prelude::stream::Stream;

        let secret = shared.read().unwrap().secret.clone();
        let server = address.connect(secret, remote_public)
            .map_err(|e| println!("{:?}", e))
            .and_then(move |stream| {
                let shared = shared.clone();
                let remote_public = stream.remote_key().clone();
                let (sink, stream) = stream.framed().split();

                let mut this = shared.write().unwrap();
                match this.add(remote_public) {
                    Some(k) => {
                        use futures::future::ok;
                        println!("WARNING: {} is connected, ignoring", k);
                        tokio::spawn(ok(()))
                    },
                    None => {
                        let peer = Remote {
                            shared: Arc::downgrade(&shared),
                            world: World::new(),
                            public: remote_public,
                        };

                        let connection = stream
                            .map_err(|e| println!("{:?}", e))
                            .fold(peer, |mut peer, message| {
                                println!("{:?}", message);
                                Ok(peer)
                            })
                            .map(|_| ());

                        tokio::spawn(connection)
                    }
                }
            });
        tokio::run(server);
    }
}
