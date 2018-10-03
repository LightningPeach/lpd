#![allow(unused_imports)]

extern crate bitcoin;
extern crate rand;
extern crate secp256k1;
extern crate brontide;
extern crate hex;
#[macro_use]
extern crate wire;
extern crate channel;
extern crate routing;
extern crate serde;

use rand::{Rng, RngCore};

use secp256k1::{SecretKey, PublicKey, Secp256k1};
use secp256k1::constants::SECRET_KEY_SIZE;
use serde::ser::Serialize;

use bitcoin::util::hash::Sha256dHash;

use std::net::{TcpStream, SocketAddr};

use std::error::Error;

use brontide::tcp_communication::{Stream, NetAddress};
use wire::{Message, BinarySD, SerdeVec, Init, RawFeatureVector, FeatureBit, Ping, Pong, AcceptChannel, OpenChannel, ChannelKeys, ChannelPrivateKeys, FundingSigned, ChannelId, FundingLocked, UpdateFulfillHtlc, UpdateAddHtlc, RevokeAndAck, CommitmentSigned};
use wire::PublicKey as LpdPublicKey;
use wire::SecretKey as LpdPrivate;
use wire::Signature as LpdSignature;

#[macro_use]
extern crate hex_literal;

use bitcoin::network::serialize::{RawEncoder};
use bitcoin::network::encodable::ConsensusEncodable;

use channel::derivation::{derive_pubkey, derive_revocation_pubkey};
use channel::tools::{get_obscuring_number, sha256};
use channel::commit::{CommitTx, HTLC, HTLCDirection};

use routing::graph::Graph;

use std::{thread, time};

fn get_key_pair() -> (SecretKey, PublicKey) {
    let ctx = Secp256k1::new();
    let sk_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
    let sk = SecretKey::from_slice(&Secp256k1::new(), &sk_bytes).unwrap();
    let pk = PublicKey::from_secret_key(&ctx, &sk).unwrap();
    return (sk,pk)
}

fn to_u8_32(u: &[u8]) -> [u8; 32] {
    assert_eq!(u.len(), 32);
    let mut rez: [u8; 32] = [0; 32];
    &rez[0..].copy_from_slice(u);
    return rez;
}

fn main() {
    // Connect to lnd node
    let ctx = Secp256k1::new();
    let local_priv_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
    let local_priv = SecretKey::from_slice(&ctx, &local_priv_bytes).unwrap();
    let local_pk = PublicKey::from_secret_key(&ctx, &local_priv).unwrap();
    println!("local_pk={}", hex::encode(&local_pk.serialize()[..]));

    let rpreimg : [u8; 32]  = rand::random();
    let rhash = sha256(&rpreimg);
    println!("rhash={}",  hex::encode(&rhash));

    let remote_pub = public_key!("0384432b67cbea5f751eb29fe8f13309c1723fb6dc37c737c4bcf017a9a38279b0");

    let socket_addr = "127.0.0.1:10000".parse().unwrap();

    let net_address = NetAddress {
        identity_key: remote_pub,
	    socket:      socket_addr,
    };

    let mut brontide_stream= Stream::connect(local_priv, net_address).unwrap();

    println!("dial: OK");

    let init_msg_req = {
        use self::FeatureBit::*;

        let global_features = RawFeatureVector::new();
        let local_features = RawFeatureVector::new().set_bit(InitialRoutingSync);
        let init = Init::new(global_features, local_features);
        Message::Init(init)
    };

    BinarySD::serialize(&mut brontide_stream, &init_msg_req).unwrap();
    println!("init_req: OK");

    let init_msg_resp: Message = BinarySD::deserialize(&mut brontide_stream).unwrap();
    println!("{:?}", init_msg_resp);

    let mut open_channel_b : Option<OpenChannel> = None;
    let mut your_commit_tx : Option<CommitTx> = None;
    let mut your_add_htlc : Option<UpdateAddHtlc> = None;

    let private_channel_keys = rand::random();
    let accept_channel_keys = ChannelKeys::new(&private_channel_keys).unwrap();

    let mut graph = Graph::new();

    let mut obscuring_factor : u64 = 0;
    loop {
        match BinarySD::deserialize(&mut brontide_stream) {
            Ok(Message::Ping(p)) =>  {
                graph.enumerate_nodes();
                println!("PING: {:?}", &p);
                let pong = Pong::new(&p);
                BinarySD::serialize(&mut brontide_stream, &Message::Pong(pong)).unwrap();
            },
            Ok(Message::OpenChannel(open_channel)) => {
                println!("OPEN_CHANNEL: {:?}", open_channel);
                println!("chain_hash: {:?}", open_channel.chain_hash);

                let accept_channel_msg = AcceptChannel::accept(&open_channel, &accept_channel_keys);
                BinarySD::serialize(&mut brontide_stream, &Message::AcceptChannel(accept_channel_msg)).unwrap();
                open_channel_b = Some(open_channel);
            },
            Ok(Message::FundingCreated(funding_created)) => {
                println!("FUNDING_CREATED: {:?}", &funding_created);
                // Now we create a commitment transaction
                // local => from OpenChannel
                // remote => from AcceptChannel

                // bit ugly, but avoid copy
                // anyway this code is temporary
                let open_channel = || {
                    match &open_channel_b {
                        &Some(ref c) => c,
                        &None => panic!(),
                    }
                };
                let open_channel_keys = || {
                    &open_channel().keys
                };

                let local_htlc_pubkey = derive_pubkey(
                    &open_channel_keys().htlc().as_ref(),
                    &open_channel_keys().first_per_commitment().as_ref(),
                );
                let remote_htlc_pubkey = derive_pubkey(
                    accept_channel_keys.htlc().as_ref(),
                    &accept_channel_keys.first_per_commitment().as_ref()
                );

                let local_revocation_pubkey = derive_revocation_pubkey(
                    &accept_channel_keys.revocation().as_ref(),
                    &open_channel_keys().first_per_commitment().as_ref()
                );
                let local_delayed_pubkey = derive_pubkey(
                    &open_channel_keys().delayed_payment().as_ref(),
                    &open_channel_keys().first_per_commitment().as_ref()
                );

                let remote_pubkey = derive_pubkey(
                    &accept_channel_keys.payment().as_ref(),
                    &accept_channel_keys.first_per_commitment().as_ref()
                );

                obscuring_factor = get_obscuring_number(
                    &open_channel_keys().payment().as_ref().serialize(),
                    &accept_channel_keys.payment().as_ref().serialize()
                );

                let commit_tx = CommitTx{
                    funding_amount: u64::from(open_channel().funding) as i64,
                    local_funding_pubkey: open_channel_keys().funding().as_ref().clone(),
                    remote_funding_pubkey: accept_channel_keys.funding().as_ref().clone(),

                    local_feerate_per_kw: u32::from(open_channel().fee) as i64,
                    dust_limit_satoshi: u64::from(open_channel().dust_limit) as i64,

                    to_local_msat: (1000 * u64::from(open_channel().funding)) as i64,
                    to_remote_msat: u64::from(open_channel().push) as i64,

                    obscured_commit_number: 0 ^ obscuring_factor,

                    local_htlc_pubkey: local_htlc_pubkey,
                    remote_htlc_pubkey: remote_htlc_pubkey,

                    local_revocation_pubkey: local_revocation_pubkey,
                    local_delayedpubkey: local_delayed_pubkey,
                    local_delay: u16::from(open_channel().csv_delay).into(),

                    remotepubkey: remote_pubkey,

                    funding_tx_id: Sha256dHash::from(&<[u8; 32]>::from(funding_created.funding_txid.clone())[..]),
                    funding_output_index: u16::from(funding_created.output_index) as u32,

                    htlcs: vec![]
                };
                let sig = commit_tx.sign(&private_channel_keys.funding_sk().as_ref());
                let tx = commit_tx.get_tx();
                let mut a = vec![];
                tx.consensus_encode(&mut RawEncoder::new(&mut a)).unwrap();
                println!("commit_tx: {}", hex::encode(a));

                println!("Signature: {:?}", sig);
                let mut channel_id = <[u8; 32]>::from(funding_created.funding_txid);
                let ind = u16::from(funding_created.output_index);
                channel_id[0] ^= (ind & 0xFF) as u8;
                channel_id[1] ^= (ind >> 8) as u8;
                let funding_signed = FundingSigned {
                    channel_id: ChannelId::from(channel_id),
                    signature: LpdSignature::from(sig),
                };
                BinarySD::serialize(&mut brontide_stream, &Message::FundingSigned(funding_signed)).unwrap();
                your_commit_tx = Some(commit_tx);
            },
            Ok(Message::FundingLocked(funding_locked)) => {
                println!("FUNDING_LOCKED: {:?}", &funding_locked);
                let (_commit_point_sk, commit_point_pk) = get_key_pair();
                let my_funding_locked = FundingLocked {
                    channel_id: funding_locked.channel_id,
                    next_per_commitment_point: LpdPublicKey::from(commit_point_pk),
                };
                BinarySD::serialize(&mut brontide_stream, &Message::FundingLocked(my_funding_locked)).unwrap();
                println!(
                    "sendpayment --dest {} --amt {} --payment_hash {} --final_cltv_delta={}",
                    hex::encode(&local_pk.serialize()[..]),
                    1,
                    hex::encode(&rhash),
                    1,
                );
            },
            Ok(Message::UpdateAddHtlc(update_add_htlc)) => {
                println!("UPDATE_ADD_HTLC: {:?}", &update_add_htlc);
                your_add_htlc = Some(update_add_htlc);
            },
            Ok(Message::CommitmentSigned(commitment_signed)) => {
                println!("COMMITMENT_SIGNED: {:?}", &commitment_signed);
                let add_htlc = your_add_htlc.unwrap();

                let (_new_commit_point_sk, new_commit_point_pk) = get_key_pair();
                println!("per_commit_point: {:?}", &accept_channel_keys.first_per_commitment());
                println!("revocation: {:?}", &private_channel_keys.first_per_commitment_sk());
                let revoke_and_ack = RevokeAndAck {
                    channel_id: commitment_signed.channel_id,
                    revocation_preimage: to_u8_32(&private_channel_keys.first_per_commitment_sk().as_ref()[..]),
                    next_per_commitment_point: LpdPublicKey::from(new_commit_point_pk),
                };
                BinarySD::serialize(&mut brontide_stream, &Message::RevokeAndAck(revoke_and_ack)).unwrap();

                let remote_pubkey = derive_pubkey(
                    &accept_channel_keys.payment().as_ref(),
                    &new_commit_point_pk
                );
                let mut commit_tx = your_commit_tx.unwrap();
                commit_tx.htlcs.push(HTLC{
                    amount_msat: u64::from(add_htlc.amount) as i64,
                    direction: HTLCDirection::Offered,
                    expiry: add_htlc.expiry as i32,
                    payment_hash: <[u8; 32]>::from(add_htlc.payment),
                });
                commit_tx.to_local_msat -= u64::from(add_htlc.amount) as i64;
                commit_tx.obscured_commit_number = 1 ^ obscuring_factor;
                commit_tx.remotepubkey = remote_pubkey;

                let tx = commit_tx.get_tx();
                let mut a = vec![];
                tx.consensus_encode(&mut RawEncoder::new(&mut a)).unwrap();
                println!("commit_tx: {}", hex::encode(&a));

                let my_commit_signed = CommitmentSigned{
                    channel_id: commitment_signed.channel_id,
                    signature: LpdSignature::from(commit_tx.sign(&private_channel_keys.funding_sk().as_ref())),
                    htlc_signatures: SerdeVec(vec![])
                };
                BinarySD::serialize(&mut brontide_stream, &Message::CommitmentSigned(my_commit_signed)).unwrap();
                your_commit_tx = Some(commit_tx);

                thread::sleep(time::Duration::from_millis(1000));

                let update_fulfill_htlc = UpdateFulfillHtlc {
                    channel_id: add_htlc.channel_id,
                    id: add_htlc.id,
                    payment_preimage: rpreimg,
                };
                BinarySD::serialize(&mut brontide_stream, &Message::UpdateFulfillHtlc(update_fulfill_htlc)).unwrap();

                your_add_htlc = Some(add_htlc);
            }
            | Ok(m @ Message::AnnouncementNode(_))
            | Ok(m @ Message::AnnouncementChannel(_))
            | Ok(m @ Message::UpdateChannel(_))
                => graph.message(m),
            Ok(msg) => println!("MSG: {:?}", msg),
            Err(_err) =>  {
//                println!("ERROR: {:?}", _err)
            }
        }
    }
}
