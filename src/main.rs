extern crate rand;
extern crate secp256k1;
extern crate brontide;
extern crate hex;

use secp256k1::{SecretKey, PublicKey, Secp256k1};
use secp256k1::constants::SECRET_KEY_SIZE;

use std::net::{TcpStream, SocketAddr};

use brontide::tcp_communication::{Stream, NetAddress};

fn main() {
    let local_priv_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
    let local_priv = SecretKey::from_slice(&Secp256k1::new(), &local_priv_bytes).unwrap();

    let public_key = PublicKey::from_secret_key(&Secp256k1::new(), &local_priv).unwrap();
    println!("{}", hex::encode(&public_key.serialize()[..]));

    let listener = brontide::tcp_communication::Listener::new(local_priv, String::from("127.0.0.1:11011")).unwrap();
    loop {
        listener.accept().unwrap();
        println!("accept");
    }

    // Connect to lnd node
//    let local_priv_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
//    let local_priv = SecretKey::from_slice(&Secp256k1::new(), &local_priv_bytes).unwrap();
//
//    let remote_pub_hex = "03f09034d318698ae6ec5e66b61b06ee5b2789fd3bce1cd62da1ba954de002a785";
//    let remote_pub_bytes = hex::decode(remote_pub_hex).unwrap();
//    let remote_pub = PublicKey::from_slice(&Secp256k1::new(), &remote_pub_bytes).unwrap();
//
//    let socket_addr = "127.0.0.1:10011".parse().unwrap();
//
//    let net_address = NetAddress {
//        identity_key: remote_pub,
//	    address:      socket_addr,
//    };
//
//    let brontide_stream= Stream::dial(local_priv, net_address).unwrap();

    println!("OK")
}