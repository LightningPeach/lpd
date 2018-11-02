extern crate rand;
extern crate secp256k1;
extern crate brontide;
extern crate tokio;
extern crate hex;

use secp256k1::{SecretKey, PublicKey, Secp256k1};
use secp256k1::constants::SECRET_KEY_SIZE;

use tokio::net;

fn main() {
    use tokio::prelude::{Stream, Future};

    let secret = {
        let local_priv_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
        SecretKey::from_slice(&Secp256k1::new(), &local_priv_bytes).unwrap()
    };

    let public = PublicKey::from_secret_key(&Secp256k1::new(), &secret).unwrap();
    let address_str = "127.0.0.1:10020";
    println!("{}@{}", hex::encode(&public.serialize()[..]), address_str);

    let listener = net::TcpListener::bind(&address_str.parse().unwrap()).ok().unwrap();

    let node_secret = secret.clone();
    let node = listener.incoming()
        .for_each(move |stream| {
            let connection = brontide::BrontideStream::incoming(stream, node_secret)
                .map(|stream| println!("new connection: {}@{}", hex::encode(&stream.remote_key().serialize()[..]), stream.as_ref().peer_addr().unwrap()))
                .map_err(|e| println!("handshake error: {:?}", e));
            tokio::spawn(connection);
            Ok(())
        })
        .map_err(|e| println!("error: {:?}", e));

    let outgoing_secret = secret.clone();
    let outgoing = net::TcpStream::connect(&"127.0.0.1:10000".parse().unwrap())
        .and_then(move |stream| {
            use secp256k1::PublicKey;
            use secp256k1::Secp256k1;
            let public = PublicKey::from_slice(&Secp256k1::new(), &hex::decode("02bb358785cba705f6339f1eca6a8209e33afc80c9207d99a90a6fbb538c668929").unwrap()).unwrap().into();
            let connection = brontide::BrontideStream::outgoing(stream, outgoing_secret, public)
                .map(|stream| println!("outgoing connection: {}", stream.as_ref().peer_addr().unwrap()))
                .map_err(|e| println!("handshake error: {:?}", e));
            tokio::spawn(connection);
            Ok(())
        })
        .map_err(|e| println!("error: {:?}", e));

    tokio::run(outgoing.and_then(|()| node));
}
