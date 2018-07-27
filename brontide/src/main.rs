extern crate rand;
extern crate secp256k1;
extern crate brontide;

use secp256k1::{SecretKey, Secp256k1};
use secp256k1::constants::SECRET_KEY_SIZE;

fn main() {
    let local_priv_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
    let local_priv = SecretKey::from_slice(&Secp256k1::new(), &local_priv_bytes).unwrap();

    let listener = brontide::tcp_communication::Listener::new(local_priv, String::from("localhost:10011"))?;
    loop {
        listener.accept();
    }

    println!("OK")
}