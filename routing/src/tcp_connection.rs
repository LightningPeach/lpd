use specs::prelude::*;
use wire::{PublicKey, SecretKey};
use std::{net, io};

#[derive(Component, Eq, PartialEq)]
pub struct Node {
    id: PublicKey,
}

#[derive(Component)]
pub struct TcpConnection {
    stream: net::TcpStream,
    address: net::SocketAddr,
    public_key: PublicKey,
}

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct TcpConnectionFailed;

/// Currently it
/// blocks the thread until new connection and
/// blocks the socket during handshake
// TODO: fixme
pub struct AddTcpConnection {
}

impl<'a> System<'a> for AddTcpConnection {
    type SystemData = (
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            update,
        ) = (&*data.0, &*data.1);

            //Ok((stream, address)) => {
            //    println!("new connection: {:?}", address);
            //    let connection_ref = entities.create();
            //    update.insert(connection_ref, Node {
            //        id: address.identity_key.clone().into(),
            //    });
            //    update.insert(connection_ref, TcpConnection {
            //        stream: stream,
            //        address: address,
            //    })
            //},
            //Err(_error) => {
            //    // TODO: collect attempts and process them somehow
            //}
    }
}
