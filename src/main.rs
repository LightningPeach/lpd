extern crate rand;
extern crate secp256k1;
extern crate brontide;
extern crate hex;
extern crate wire;
extern crate serde;

use rand::{Rng, RngCore};

use secp256k1::{SecretKey, PublicKey, Secp256k1};
use secp256k1::constants::SECRET_KEY_SIZE;
use serde::ser::Serialize;

use std::net::{TcpStream, SocketAddr};

use std::error::Error;

use brontide::tcp_communication::{Stream, NetAddress};
use wire::{Message, BinarySD, Init, RawFeatureVector, FeatureBit, Ping, Pong, AcceptChannel};
use wire::PublicKey as LpdPublicKey;

fn write_msg(stream: &mut Stream, msg: &Message) -> Result<(), Box<Error>> {
    let mut data = Vec::<u8>::new();
    BinarySD::serialize(&mut data, msg)?;
    stream.encrypt_and_write_message(&data)?;
    Ok(())
}

fn read_msg(stream: &mut Stream) -> Result<Message, Box<Error>> {
    let data = stream.read_and_decrypt_message()?;
    let msg = BinarySD::deserialize(&data[..])?;
    Ok(msg)
}

fn get_key_pair() -> (SecretKey, PublicKey) {
    let ctx = Secp256k1::new();
    let sk_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
    let sk = SecretKey::from_slice(&Secp256k1::new(), &sk_bytes).unwrap();
    let pk = PublicKey::from_secret_key(&ctx, &sk).unwrap();
    return (sk,pk)
}

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

    let remote_pub_hex = "02968e42dcc075d6c747b46688eb2527bf549b000459bc9c1bc45d762a0c0c38d1";
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

    write_msg(&mut brontide_stream, &init_msg_req).unwrap();
    println!("init_req: OK");

    let init_msg_resp = read_msg(&mut brontide_stream).unwrap();
    println!("{:?}", init_msg_resp);

    while true {
        match read_msg(&mut brontide_stream) {
            Ok(Message::Ping(p)) =>  {
                println!("PING: {:?}", &p);
                let pong = Pong::new(&p);
                write_msg(&mut brontide_stream, &Message::Pong(pong)).unwrap();
            },
            Ok(Message::OpenChannel(open_channel)) => {
                println!("OPEN_CHANNEL: {:?}", open_channel);
                println!("chain_hash: {:?}", open_channel.chain_hash);
                let (funding_sk, funding_pk) = get_key_pair();
                let (revocation_sk, revocation_pk) = get_key_pair();
                let (payment_sk, payment_pk) = get_key_pair();
                let (delayed_payment_sk, delayed_payment_pk) = get_key_pair();
                let (htlc_sk, htlc_pk) = get_key_pair();
                let (first_per_commitment_sk, first_per_commitment_pk) = get_key_pair();

                let accept_channel_msg = AcceptChannel {
                    temporary_channel_id: open_channel.temporary_channel_id,
                    dust_limit: open_channel.dust_limit.clone(),
                    max_htlc_value_in_flight: open_channel.max_in_flight.clone(),
                    chanel_reserve: open_channel.channel_reserve.clone(),
                    htlc_minimum: open_channel.htlc_minimum.clone(),
                    minimum_accept_depth: 1,
                    csv_delay: open_channel.csv_delay.clone(),
                    max_accepted_htlc_number: open_channel.max_accepted_htlc_number.clone(),
                    funding_pubkey: LpdPublicKey::from(funding_pk),
                    revocation_point: LpdPublicKey::from(revocation_pk),
                    payment_point: LpdPublicKey::from(payment_pk),
                    delayed_payment_point: LpdPublicKey::from(delayed_payment_pk),
                    htlc_point: LpdPublicKey::from(htlc_pk),
                    first_per_commitment_point: LpdPublicKey::from(first_per_commitment_pk),
                };
                write_msg(&mut brontide_stream, &Message::AcceptChannel(accept_channel_msg)).unwrap();
            },
            Ok(msg) => println!("MSG: {:?}", msg),
            Err(err) => println!("ERROR: {:?}", err)
        }
    }

}
