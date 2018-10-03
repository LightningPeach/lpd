extern crate rand;
extern crate secp256k1;
extern crate brontide;

use secp256k1::{SecretKey, Secp256k1};
use secp256k1::constants::SECRET_KEY_SIZE;

use std::net;

fn main() {
    let local_priv_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
    let local_priv = SecretKey::from_slice(&Secp256k1::new(), &local_priv_bytes).unwrap();

    let listener = net::TcpListener::bind("localhost:10011".to_owned()).ok().unwrap();
    loop {
        let (_, net_address) = brontide::tcp_communication::Stream::accept(&listener, local_priv).ok().unwrap();
        println!("{:?}", net_address);
    }
}
