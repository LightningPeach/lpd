extern crate rand;
extern crate secp256k1;
extern crate brontide;
extern crate hex;
extern crate wire;
extern crate serde;

use secp256k1::{SecretKey, PublicKey, Secp256k1};
use secp256k1::constants::SECRET_KEY_SIZE;

use std::net::{TcpStream, SocketAddr};

use brontide::tcp_communication::{Stream, NetAddress};
use wire::{Message, BinarySD, Init, RawFeatureVector, FeatureBit, Ping};

fn main() {
//    let local_priv_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
//    let local_priv = SecretKey::from_slice(&Secp256k1::new(), &local_priv_bytes).unwrap();
//
//    let public_key = PublicKey::from_secret_key(&Secp256k1::new(), &local_priv).unwrap();
//    println!("{}", hex::encode(&public_key.serialize()[..]));
//
//    let listener = brontide::tcp_communication::Listener::new(local_priv, String::from("127.0.0.1:11011")).unwrap();
//    loop {
//        listener.accept().unwrap();
//        println!("accept");
//    }

    // Connect to lnd node
    let local_priv_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
    let local_priv = SecretKey::from_slice(&Secp256k1::new(), &local_priv_bytes).unwrap();

    let remote_pub_hex = "0321a4a0c88e3bd2b757972eeda45673f624a504d7e14eb86c23922706668711e4";
    let remote_pub_bytes = hex::decode(remote_pub_hex).unwrap();
    let remote_pub = PublicKey::from_slice(&Secp256k1::new(), &remote_pub_bytes).unwrap();

    let socket_addr = "127.0.0.1:10011".parse().unwrap();

    let net_address = NetAddress {
        identity_key: remote_pub,
	    address:      socket_addr,
    };

    let mut brontide_stream= Stream::dial(local_priv, net_address).unwrap();

    println!("dial: OK");

    let init_msg_req = {
        use self::FeatureBit::*;

        let global_features = RawFeatureVector::new();
        let local_features = RawFeatureVector::new().set_bit(InitialRoutingSync);
        let init = Init::new(global_features, local_features);
        Message::Init(init)
    };

    let mut data = Vec::<u8>::new();
    BinarySD::serialize(&mut data, &init_msg_req).unwrap();
    println!("init_req: {:?}", data);

    brontide_stream.encrypt_and_write_message(&data).unwrap();
    println!("init_req: OK");

    let raw_init_msg_resp = brontide_stream.read_and_decrypt_message().unwrap();
    println!("init_resp: OK");

    println!("{}", raw_init_msg_resp.len());
    println!("{:?}", raw_init_msg_resp);

    let init_msg_resp: Message = BinarySD::deserialize(&raw_init_msg_resp[..]).unwrap();
    println!("{:?}", init_msg_resp);

    let mut pong = |pong_length: u16| {
        let ping = Ping::new(16, pong_length)?;
        let mut data = Vec::<u8>::new();
        BinarySD::serialize(&mut data, &Message::Ping(ping)).map_err(|_| ())?;
        brontide_stream.encrypt_and_write_message(&data).map_err(|_| ())?;
        data = brontide_stream.read_and_decrypt_message().map_err(|_| ())?;
        match BinarySD::deserialize(&data[..]).map_err(|_| ())? {
            Message::Pong(pong) => {
                if pong.length() == pong_length {
                    Ok(pong)
                } else {
                    Err(())
                }
            },
            _ => Err(())
        }
    };

    let q = (0..8).into_iter().map(|i| pong(16 + i).unwrap()).collect::<Vec<_>>();
    println!("{:?}", q)
}
