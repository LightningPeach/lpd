extern crate bitcoin;
extern crate rand;
extern crate secp256k1;
extern crate brontide;
extern crate hex;
#[macro_use]
extern crate wire;
extern crate channel;
extern crate routing;
extern crate tokio;
extern crate futures;

use secp256k1::{SecretKey, PublicKey, Secp256k1};
use secp256k1::constants::SECRET_KEY_SIZE;

use bitcoin::util::hash::Sha256dHash;

use std::net::SocketAddr;

use brontide::{BrontideStream, Machine};
use wire::{
    Message, SerdeVec, Init, Ping, Pong, AcceptChannel, ChannelKeys, ChannelPrivateKeys,
    OpenChannel, FundingSigned, FundingCreated, ChannelId, FundingLocked,
    UpdateFulfillHtlc, UpdateAddHtlc, RevokeAndAck, CommitmentSigned,
    MessageConsumer, WireError, MessageFiltered, MessageConsumerChain
};
use wire::PublicKey as LpdPublicKey;
use wire::Signature as LpdSignature;

#[macro_use]
extern crate hex_literal;

use bitcoin::network::serialize::{RawEncoder};
use bitcoin::network::encodable::ConsensusEncodable;

use channel::derivation::{derive_pubkey, derive_revocation_pubkey};
use channel::tools::{get_obscuring_number, sha256};
use channel::commit::{CommitTx, HTLC, HTLCDirection};

use routing::Graph;

use std::{thread, time, cell};

use tokio::net;
use tokio::prelude::Future;
use tokio::prelude::Sink;
use tokio::prelude::Stream;
use tokio::codec::Framed;

pub struct PingMessage {
    inner: Ping,
}

impl MessageFiltered for PingMessage {
    fn filter(v: Message) -> Result<Self, Message> {
        match v {
            Message::Ping(p) => Ok(PingMessage { inner: p }),
            v @ _ => Err(v),
        }
    }
}

pub struct PingResponder;

impl MessageConsumer for PingResponder {
    type Message = PingMessage;

    fn consume<S>(self, sink: S, message: Self::Message) -> Box<dyn Future<Item=(Self, S), Error=WireError>>
    where
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        let p = message.inner;
        println!("PING: {:?}", &p);
        let pong = Pong::new(&p);
        Box::new(
            sink.send(Message::Pong(pong))
                .map(move |s| (self, s))
        )
    }
}

pub enum MainMessage {
    OpenChannel(OpenChannel),
    FundingCreated(FundingCreated),
    FundingLocked(FundingLocked),
    UpdateAddHtlc(UpdateAddHtlc),
    CommitmentSigned(CommitmentSigned),
}

impl MessageFiltered for MainMessage {
    fn filter(v: Message) -> Result<Self, Message> {
        match v {
            Message::OpenChannel(v) => Ok(MainMessage::OpenChannel(v)),
            Message::FundingCreated(v) => Ok(MainMessage::FundingCreated(v)),
            Message::FundingLocked(v) => Ok(MainMessage::FundingLocked(v)),
            Message::UpdateAddHtlc(v) => Ok(MainMessage::UpdateAddHtlc(v)),
            Message::CommitmentSigned(v) => Ok(MainMessage::CommitmentSigned(v)),
            v @ _ => Err(v)
        }
    }
}

pub struct MainContext {
    rpreimg: [u8; 32],
    open_channel_b: Option<OpenChannel>,
    your_commit_tx: Option<CommitTx>,
    your_add_htlc: Option<UpdateAddHtlc>,
    channel_secret_keys: ChannelPrivateKeys,
    channel_keys: ChannelKeys,
    obscuring_factor: cell::Cell<u64>,
}

impl MessageConsumer for MainContext {
    type Message = MainMessage;

    fn consume<S>(mut self, sink: S, message: Self::Message) -> Box<dyn Future<Item=(Self, S), Error=WireError>>
    where
        S: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
    {
        use tokio::prelude::IntoFuture;
        use wire::MilliSatoshi;

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

        match message {
            MainMessage::OpenChannel(open_channel) => {
                println!("OPEN_CHANNEL: {:?}", open_channel);
                println!("chain_hash: {:?}", open_channel.chain_hash);

                let accept_channel_msg = AcceptChannel::accept(&open_channel, &self.channel_keys);
                Box::new(
                    sink.send(Message::AcceptChannel(accept_channel_msg))
                        .map(move |s| {
                            self.open_channel_b = Some(open_channel);
                            (self, s)
                        })
                )
            },
            MainMessage::FundingCreated(funding_created) => {
                println!("FUNDING_CREATED: {:?}", &funding_created);
                // Now we create a commitment transaction
                // local => from OpenChannel
                // remote => from AcceptChannel

                let local_htlc_pubkey = derive_pubkey(
                    &self.open_channel_b.as_ref().unwrap().keys.htlc().as_ref(),
                    &self.open_channel_b.as_ref().unwrap().keys.first_per_commitment().as_ref(),
                );
                let remote_htlc_pubkey = derive_pubkey(
                    self.channel_keys.htlc().as_ref(),
                    &self.channel_keys.first_per_commitment().as_ref()
                );

                let local_revocation_pubkey = derive_revocation_pubkey(
                    &self.channel_keys.revocation().as_ref(),
                    &self.open_channel_b.as_ref().unwrap().keys.first_per_commitment().as_ref()
                );
                let local_delayed_pubkey = derive_pubkey(
                    &self.open_channel_b.as_ref().unwrap().keys.delayed_payment().as_ref(),
                    &self.open_channel_b.as_ref().unwrap().keys.first_per_commitment().as_ref()
                );

                let remote_pubkey = derive_pubkey(
                    &self.channel_keys.payment().as_ref(),
                    &self.channel_keys.first_per_commitment().as_ref()
                );

                self.obscuring_factor.set(get_obscuring_number(
                    &self.open_channel_b.as_ref().unwrap().keys.payment().as_ref().serialize(),
                    &self.channel_keys.payment().as_ref().serialize()
                ));

                let commit_tx = CommitTx {
                    funding_amount: u64::from(self.open_channel_b.as_ref().unwrap().funding) as i64,
                    local_funding_pubkey: self.open_channel_b.as_ref().unwrap().keys.funding().as_ref().clone(),
                    remote_funding_pubkey: self.channel_keys.funding().as_ref().clone(),

                    local_feerate_per_kw: u32::from(self.open_channel_b.as_ref().unwrap().fee) as i64,
                    dust_limit_satoshi: u64::from(self.open_channel_b.as_ref().unwrap().dust_limit) as i64,

                    to_local_msat: (1000 * u64::from(self.open_channel_b.as_ref().unwrap().funding)) as i64,
                    to_remote_msat: u64::from(self.open_channel_b.as_ref().unwrap().push) as i64,

                    obscured_commit_number: 0 ^ self.obscuring_factor.get(),

                    local_htlc_pubkey: local_htlc_pubkey,
                    remote_htlc_pubkey: remote_htlc_pubkey,

                    local_revocation_pubkey: local_revocation_pubkey,
                    local_delayedpubkey: local_delayed_pubkey,
                    local_delay: u16::from(self.open_channel_b.as_ref().unwrap().csv_delay).into(),

                    remotepubkey: remote_pubkey,

                    funding_tx_id: Sha256dHash::from(&<[u8; 32]>::from(funding_created.funding_txid.clone())[..]),
                    funding_output_index: u16::from(funding_created.output_index) as u32,

                    htlcs: vec![]
                };
                let sig = commit_tx.sign(&self.channel_secret_keys.funding_sk().as_ref());
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
                Box::new(
                    sink.send(Message::FundingSigned(funding_signed))
                        .map(move |s| {
                            self.your_commit_tx = Some(commit_tx);
                            (self, s)
                       })
                )
            },
            MainMessage::FundingLocked(funding_locked) => {
                println!("FUNDING_LOCKED: {:?}", &funding_locked);
                let (_commit_point_sk, commit_point_pk) = get_key_pair();
                let my_funding_locked = FundingLocked {
                    channel_id: funding_locked.channel_id,
                    next_per_commitment_point: LpdPublicKey::from(commit_point_pk),
                };
                Box::new(
                    sink.send(Message::FundingLocked(my_funding_locked))
                        .map(move |s| {
                            //println!(
                            //    "sendpayment --dest {} --amt {} --payment_hash {} --final_cltv_delta={}",
                            //    hex::encode(&local_pk.serialize()[..]),
                            //    1,
                            //    hex::encode(&rhash),
                            //    1,
                            //);
                            (self, s)
                        })
                )
            },
            MainMessage::UpdateAddHtlc(update_add_htlc) => {
                println!("UPDATE_ADD_HTLC: {:?}", &update_add_htlc);
                self.your_add_htlc = Some(update_add_htlc);
                Box::new(Ok((self, sink)).into_future())
            },
            MainMessage::CommitmentSigned(commitment_signed) => {
                println!("COMMITMENT_SIGNED: {:?}", &commitment_signed);

                let (_new_commit_point_sk, new_commit_point_pk) = get_key_pair();
                println!("per_commit_point: {:?}", &self.channel_keys.first_per_commitment());
                println!("revocation: {:?}", &self.channel_secret_keys.first_per_commitment_sk());
                let revoke_and_ack = RevokeAndAck {
                    channel_id: commitment_signed.channel_id,
                    revocation_preimage: to_u8_32(&self.channel_secret_keys.first_per_commitment_sk().as_ref()[..]),
                    next_per_commitment_point: LpdPublicKey::from(new_commit_point_pk),
                };
                Box::new(
                    sink
                        .send(Message::RevokeAndAck(revoke_and_ack))
                        .map_err(|e| panic!("error: {:?}", e))
                        .and_then(move |sink| {
                            let remote_pubkey = derive_pubkey(
                                &self.channel_keys.payment().as_ref(),
                                &new_commit_point_pk
                            );
                            let my_commit_signed = {
                                let add_htlc = self.your_add_htlc.as_ref().unwrap();
                                let mut commit_tx = self.your_commit_tx.as_mut().unwrap();
                                commit_tx.htlcs.push(HTLC {
                                    amount_msat: <u64 as From<MilliSatoshi>>::from(add_htlc.amount) as i64,
                                    direction: HTLCDirection::Offered,
                                    expiry: add_htlc.expiry as i32,
                                    payment_hash: <[u8; 32]>::from(add_htlc.payment),
                                });
                                commit_tx.to_local_msat -= <u64 as From<MilliSatoshi>>::from(add_htlc.amount) as i64;
                                commit_tx.obscured_commit_number = 1 ^ self.obscuring_factor.get();
                                commit_tx.remotepubkey = remote_pubkey;

                                let tx = commit_tx.get_tx();
                                let mut a = vec![];
                                tx.consensus_encode(&mut RawEncoder::new(&mut a)).unwrap();
                                println!("commit_tx: {}", hex::encode(&a));

                                CommitmentSigned{
                                    channel_id: commitment_signed.channel_id,
                                    signature: LpdSignature::from(commit_tx.sign(&self.channel_secret_keys.funding_sk().as_ref())),
                                    htlc_signatures: SerdeVec(vec![])
                                }
                            };
                            sink.send(Message::CommitmentSigned(my_commit_signed))
                                .and_then(move |sink| {
                                    thread::sleep(time::Duration::from_millis(1000));

                                    let update_fulfill_htlc = {
                                        let add_htlc = self.your_add_htlc.as_ref().unwrap();
                                        UpdateFulfillHtlc {
                                            channel_id: add_htlc.channel_id,
                                            id: add_htlc.id,
                                            payment_preimage: self.rpreimg,
                                        }
                                    };
                                    sink.send(Message::UpdateFulfillHtlc(update_fulfill_htlc))
                                        .map(move |sink| (self, sink))
                                })
                        })
                )
            },
        }
    }
}

impl MainContext {
    pub fn new() -> Self {
        let private_channel_keys = rand::random();
        let accept_channel_keys = ChannelKeys::new(&private_channel_keys).unwrap();

        let rpreimg : [u8; 32]  = rand::random();
        let rhash = sha256(&rpreimg);
        println!("rhash={}",  hex::encode(&rhash));

        MainContext {
            rpreimg: rpreimg,
            open_channel_b: None,
            your_commit_tx: None,
            your_add_htlc: None,
            channel_secret_keys: private_channel_keys,
            channel_keys: accept_channel_keys,
            obscuring_factor: cell::Cell::new(0),
        }
    }
}

fn connect(secret_key: SecretKey, remote_address: &SocketAddr, remote_key: PublicKey) -> impl Future<Item=Framed<net::TcpStream, Box<Machine>>, Error=()> {
    net::TcpStream::connect(&remote_address)
        .map_err(|e| panic!("error: {:?}", e))
        .and_then(move |stream|
            BrontideStream::outgoing(stream, secret_key, remote_key)
                .map_err(|e| panic!("error: {:?}", e))
                .map(BrontideStream::framed)
        )
        .and_then(|stream| {
            let init_msg_req = {
                use wire::FeatureBit::*;
                use wire::RawFeatureVector;

                let global_features = RawFeatureVector::new();
                let local_features = RawFeatureVector::new().set_bit(InitialRoutingSync);
                let init = Init::new(global_features, local_features);
                Message::Init(init)
            };
            stream
                .send(init_msg_req)
                .map_err(|e| panic!("error: {:?}", e))
        })
}

fn process<I, O>(stream: I, sink: O) -> impl Future<Item=(), Error=()>
where
    I: Stream<Item=Message, Error=WireError>,
    O: Sink<SinkItem=Message, SinkError=WireError> + Send + 'static,
{
    use tokio::prelude::IntoFuture;

    let contexts = (PingResponder, (Graph::new(), (MainContext::new(), ())));
    stream
        .fold((contexts, sink), |(contexts, sink), message| {
            contexts.process(sink, message)
                // if any previous MessageConsumer did not consumed the message
                .or_else::<(), _>(|(contexts, sink, message)| {
                    println!("warning: skipped message {:?}", message);
                    // always Ok, so could unwrap
                    Ok(Box::new(Ok((contexts, sink)).into_future()))
                })
                .ok().unwrap()
        })
        .map(|(_, _)| ())
        .map_err(|e| panic!("error: {:?}", e))
}

fn main() {
    use tokio::runtime::current_thread;
    use futures::future;

    // Connect to lnd node
    let ctx = Secp256k1::new();
    let local_priv_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
    let local_private = SecretKey::from_slice(&ctx, &local_priv_bytes).unwrap();
    let local_public = PublicKey::from_secret_key(&ctx, &local_private).unwrap();
    println!("local_pk={}", hex::encode(&local_public.serialize()[..]));

    let address = "127.0.0.1:10100".parse().unwrap();
    tokio::run(net::TcpListener::bind(&address)
        .unwrap()
        .incoming()
        .for_each(move |stream| {
            BrontideStream::incoming(stream, local_private)
                .map(|stream| {
                    println!("incoming: {:?}", stream.remote_key());
                })
                .map_err(|e| { panic!("error: {:?}", e); unimplemented!() })
        })
        .map_err(|e| panic!("error: {:?}", e))
    );

    let remote_pub = public_key!("02050883052b49e6cf63ed6e7de10bf419d7c846c989af57d817c7471d37a29586");
    let address = "127.0.0.1:10000".parse().unwrap();
    let task = connect(local_private, &address, remote_pub)
        .and_then(move |s| {
            let (sink, stream) = s.split();
            process(stream, sink)
        });
    current_thread::block_on_all(future::lazy(|| {
        current_thread::spawn(task);
        Ok::<_, ()>(())
    })).unwrap();
}
