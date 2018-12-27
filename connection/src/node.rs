use std::sync::{Arc, RwLock};

use secp256k1::{SecretKey, PublicKey};
use tokio::prelude::{Future, AsyncRead, AsyncWrite};
use tokio::executor::Spawn;
use futures::sync::mpsc::Receiver;
use specs::{World, System};
use wire::Message;

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
    world: World,
    // TODO: store as entity in the world
    peers: Vec<PublicKey>,
    secret: SecretKey,
}

pub struct Remote {
    local: World,
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
            peers: Vec::new(),
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
        use specs::RunNow;

        let remote_public = stream.remote_key().clone();
        let (sink, stream) = stream.framed().split();

        // nll will fix it,
        // p_self is borrowed, but it is not used in match's `None` arm
        // however, the compiler still complains, so let's clone the pointer here
        let _shared = p_self.clone();
        match p_self.write().unwrap().add(remote_public) {
            Some(k) => {
                use futures::future::ok;
                println!("WARNING: {} is connected, ignoring", k);
                tokio::spawn(ok(()))
            },
            None => {
                println!("INFO: new peer {:?}", hex::encode(&remote_public.serialize()[..]));

                let peer = Remote {
                    local: {
                        let mut world = World::new();
                        // world.register();
                        world
                    },
                    public: remote_public,
                };

                let connection = stream
                    .map_err(|e| println!("{:?}", e))
                    .fold(peer, move |mut peer, message| {
                        println!("{:?}", message);
                        Msg(Some(message)).run_now(&mut peer.local.res);
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
}
