extern crate bitcoin;
extern crate rand;
extern crate secp256k1;
extern crate brontide;
extern crate hex;
extern crate wire;
extern crate channel;
extern crate serde;

use rand::{Rng, RngCore};

use secp256k1::{SecretKey, PublicKey, Secp256k1};
use secp256k1::constants::SECRET_KEY_SIZE;
use serde::ser::Serialize;

use bitcoin::util::hash::Sha256dHash;

use std::net::{TcpStream, SocketAddr};

use std::error::Error;

use brontide::tcp_communication::{Stream, NetAddress};
use wire::{Message, BinarySD, SerdeVec, Init, RawFeatureVector, FeatureBit, Ping, Pong, AcceptChannel, OpenChannel, FundingSigned, ChannelId, FundingLocked, UpdateFulfillHtlc, UpdateAddHtlc, RevokeAndAck, CommitmentSigned, Satoshi, MilliSatoshi, CsvDelay, FundingCreated};
use wire::PublicKey as LpdPublicKey;
use wire::Signature as LpdSignature;

use bitcoin::network::serialize::{RawEncoder};
use bitcoin::network::encodable::ConsensusEncodable;

use channel::derivation::{derive_pubkey, derive_revocation_pubkey};
use channel::tools::{get_obscuring_number, sha256};
use channel::commit::{CommitTx, HTLC, HTLCDirection};

use std::{thread, time};

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

// TODO(mkl): add an option to generate keys from a seed
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
    for i in 0..32 {
        rez[i] = u[i];
    }
    return rez;
}

// This parameters can be reconfigured
// so they is in a separate structure
// TODO(mkl): compare with updated documentation (like BOLT 1.1)
#[derive(Debug, Copy, Clone, Default)]
struct PartnerConfig {
    dust_limit: u64,
    max_htlc_value_in_flight: u64,
    chanel_reserve: u64,
    htlc_minimum: u64,
    csv_delay: u16,
    max_accepted_htlc_number: u16,
    local_fee_rate: u32,
}

// bp means "base point"
// base point is public or private key depending on context
// e.g. revocation_bp_sk - revocation basepoint secret(private) key
// revocation_bp_pk - revocation basepoint public key
#[derive(Debug, Clone)]
struct PartnerInfo {
    funding_sk: Option<SecretKey>,
    funding_pk: PublicKey,
    revocation_bp_sk: Option<SecretKey>,
    revocation_bp_pk: PublicKey,
    payment_bp_sk: Option<SecretKey>,
    payment_bp_pk: PublicKey,
    delayed_payment_bp_sk: Option<SecretKey>,
    delayed_payment_bp_pk: PublicKey,
    htlc_bp_sk: Option<SecretKey>,
    htlc_bp_pk: PublicKey,
    per_commit_sk: Option<SecretKey>,
    per_commit_pk: PublicKey,

    config: PartnerConfig,
    // TODO(mkl): add flag to indicate if info contains private info
    // TODO(mkl): add flag to indicate if it is an initiator info
}

#[derive(Debug, Default, Copy, Clone)]
struct FundingInfo {
    temporary_channel_id: ChannelId,
    funding: u64,
    push: u64,
}

impl FundingInfo {
    fn from_open_channel_msg(msg: &OpenChannel) -> FundingInfo {
        FundingInfo {
            temporary_channel_id: msg.temporary_channel_id,
            funding: u64::from(msg.funding),
            push: u64::from(msg.push)
        }
    }
}

impl PartnerInfo {
    // Create new PartnerInfo with random info
    fn new_random() -> PartnerInfo {
        let (funding_sk, funding_pk) = get_key_pair();
        let (revocation_sk, revocation_pk) = get_key_pair();
        let (payment_sk, payment_pk) = get_key_pair();
        let (delayed_payment_sk, delayed_payment_pk) = get_key_pair();
        let (htlc_sk, htlc_pk) = get_key_pair();
        let (first_per_commitment_sk, first_per_commitment_pk) = get_key_pair();
        PartnerInfo {
            funding_sk: Some(funding_sk),
            funding_pk: funding_pk,
            revocation_bp_sk: Some(revocation_sk),
            revocation_bp_pk: revocation_pk,
            payment_bp_sk: Some(payment_sk),
            payment_bp_pk: payment_pk,
            delayed_payment_bp_sk: Some(delayed_payment_sk),
            delayed_payment_bp_pk: delayed_payment_pk,
            htlc_bp_sk: Some(htlc_sk),
            htlc_bp_pk: htlc_pk,
            per_commit_sk: Some(first_per_commitment_sk),
            per_commit_pk: first_per_commitment_pk,

            config: Default::default()
        }
    }

    fn from_open_channel_msg(msg: &OpenChannel) -> PartnerInfo {
        let config = PartnerConfig{
            dust_limit: u64::from(msg.dust_limit),
            max_htlc_value_in_flight: u64::from(msg.max_in_flight),
            chanel_reserve: u64::from(msg.channel_reserve),
            htlc_minimum: u64::from(msg.htlc_minimum),
            csv_delay: u16::from(msg.csv_delay),
            max_accepted_htlc_number: u16::from(msg.max_accepted_htlc_number),
            local_fee_rate: u32::from(msg.fee)
        };

        PartnerInfo {
            funding_sk: None,
            funding_pk: PublicKey::from(msg.funding_pubkey),
            revocation_bp_sk: None,
            revocation_bp_pk: PublicKey::from(msg.revocation_basepoint),
            payment_bp_sk: None,
            payment_bp_pk: PublicKey::from(msg.payment_basepoint),
            delayed_payment_bp_sk: None,
            delayed_payment_bp_pk: PublicKey::from(msg.delayed_payment_basepoint),
            htlc_bp_sk: None,
            htlc_bp_pk: PublicKey::from(msg.htlc_basepoint),
            per_commit_sk: None,
            per_commit_pk: PublicKey::from(msg.first_per_commitment_point),

            config
        }
    }

    fn htlc_pubkey(&self) -> PublicKey {
        derive_pubkey(&self.htlc_bp_pk, &self.per_commit_pk)
    }

    // Revocation pubkey for our commitment transaction
    // We require partners revocation_pk to generate it
    // and our commitment point
    fn revocation_pubkey(&self, partner_revocation_pk: &PublicKey) -> PublicKey {
        derive_revocation_pubkey(partner_revocation_pk, &self.per_commit_pk)
    }

    // We use this pubkey to send money to ourself
    fn delayed_pubkey(&self) -> PublicKey {
        derive_pubkey(&self.delayed_payment_bp_pk, &self.per_commit_pk)
    }

    fn  payment_pubkey(&self) -> PublicKey {
        derive_pubkey(&self.payment_bp_pk, &self.per_commit_pk)
    }
}

#[derive(Debug, Clone)]
struct InitialState ();

// TODO(mkl): refactor into enum
#[derive(Debug, Clone)]
struct ReadyState {
    channel_id: ChannelId,
    our_info: PartnerInfo,
    their_info: PartnerInfo,
    obscuring_factor: u64,
    funding_tx_id: Sha256dHash,
    funding_output_index: u16,
}


#[derive(Debug, Clone)]
enum ChannelState {

    // Initial state of the system. No channels
    Initial(InitialState),

    // When opening channel
    Opening(OpeningState),

    // When channel operates
    Ready(ReadyState),

    // When channel in process of cooperative closing
    Closing,

    // When channel in process of not cooperative closing
    ForceClosing,

    // When channel is closed.
    // TODO: maybe split in two cooperative and not-cooperative
    Closed,

    // When error occured during channel operation
    Error,

    // When we detect that partner is cheating
    // and start punishing him
    PunishingPartner,

    // When we successfully punish partner
    // for his cheating
    PunishedPartner,

    // When partner successfully cheat on us
    Robbed,

    // When we try to cheat (or restore from old backup)
    // and get punished
    PunishedByPartner,
}

// UpdateInfo represents update applied to channel
enum UpdateInfo {
    AddHtlc {
        id: i64,
        amount: u64,
        payment_hash: [u8; 32],
        expiry: u32,
    },
    FulfillHtlc {
        id: i64,
        payment_preimage: [u8; 32]
    },
    FailHtlc {
        id: i64,
        reason: Vec<u8>,
    }
}

struct Channel {
    our_info: PartnerInfo,
    their_info: PartnerInfo,
    funding: FundingInfo,
// TODO: add our information
    //    our_commit_tx: CommitTx,
    their_commit_tx: Option<CommitTx>
}

// Channel opening. We are receiving channel
// Partner --- OpenChannel   --->  We
// Partner <-- AcceptChannel ---   We
// Partner --- FundingCreated -->  We
// Partner <-- FundingSigned ---   We
// ...... after blockchain confirmations
// Partner <-- FundingLocked --- We
// Partner --- FundingLocked ---> We

// Data for opening channel state
// and channel is opened by other side
// When we already sent AcceptChannel message and now
// waiting for FundingCreated message
#[derive(Debug, Clone)]
struct WaitFundingCreatedData {
    temp_channel_id: ChannelId,
    our_info: PartnerInfo,
    their_info: PartnerInfo,
    funding: FundingInfo,
}

// Data for opening channel state
// and channel is opened by other side
// When we already sent FundingSigned message
// and now wait for FundingLocked
#[derive(Debug, Clone)]
struct WaitFundingLockedData {
    temp_channel_id: ChannelId,
    channel_id: ChannelId,
    our_info: PartnerInfo,
    their_info: PartnerInfo,
    obscuring_factor: u64,
    funding: FundingInfo,
    funding_tx_id: Sha256dHash,
    funding_output_index: u16,
}

#[derive(Debug, Clone)]
enum OpeningState {
    Initial,
    WaitFundingCreated(WaitFundingCreatedData),
    WaitFundingLocked(WaitFundingLockedData),
    Error(String)
}

// TODO(mkl): add states for us opening channels

#[derive(Debug, Clone)]
enum Event {
    MessageEvent(Message),
    ControlEvent,
    BlockchainEvent,
    TimerEvent,
}

impl ChannelState {
    fn new() -> ChannelState {
        ChannelState::Initial(InitialState())
    }

    fn next(self, msg: Message) -> (ChannelState, Option<Message>) {
        match (self, msg) {
            (ChannelState::Initial(st), Message::OpenChannel(msg)) => {
                st.handle_open_channel_msg(msg)
            },
            (ChannelState::Opening(OpeningState::WaitFundingCreated(st)), Message::FundingCreated(msg)) => {
                st.handle_funding_created_msg(msg)
            },
            (ChannelState::Opening(OpeningState::WaitFundingLocked(st)), Message::FundingLocked(msg)) => {
                st.handle_funding_locked_msg(msg)
            },
            (st, msg) => {
                println!("Unknown combination state/message: {:?}/{:?}", &st, &msg);
                (st, None)
            }
        }
    }
}

impl InitialState {
    fn handle_open_channel_msg(self, msg: OpenChannel) -> (ChannelState, Option<Message>) {
        let their_info = PartnerInfo::from_open_channel_msg(&msg);
        let mut our_info = PartnerInfo::new_random();
        our_info.config = their_info.config.clone();
        let funding = FundingInfo::from_open_channel_msg(&msg);
        let accept_channel_msg = AcceptChannel {
            temporary_channel_id: msg.temporary_channel_id.clone(),
            dust_limit: Satoshi::from(their_info.config.dust_limit),
            max_htlc_value_in_flight: MilliSatoshi::from(their_info.config.max_htlc_value_in_flight),
            chanel_reserve: Satoshi::from(their_info.config.chanel_reserve),
            htlc_minimum: MilliSatoshi::from(their_info.config.htlc_minimum),
            minimum_accept_depth: 1,
            csv_delay: CsvDelay::from(their_info.config.csv_delay),
            max_accepted_htlc_number: their_info.config.max_accepted_htlc_number,
            funding_pubkey: LpdPublicKey::from(our_info.funding_pk),
            revocation_point: LpdPublicKey::from(our_info.revocation_bp_pk),
            payment_point: LpdPublicKey::from(our_info.payment_bp_pk),
            delayed_payment_point: LpdPublicKey::from(our_info.delayed_payment_bp_pk),
            htlc_point: LpdPublicKey::from(our_info.htlc_bp_pk),
            first_per_commitment_point: LpdPublicKey::from(our_info.per_commit_pk),
        };
        let data = WaitFundingCreatedData {
            our_info,
            their_info,
            temp_channel_id: msg.temporary_channel_id.into(),
            funding: FundingInfo::from_open_channel_msg(&msg)
        };
        (
            ChannelState::Opening(OpeningState::WaitFundingCreated(data)),
            Some(Message::AcceptChannel(accept_channel_msg))
        )
    }
}

impl WaitFundingCreatedData {
    fn handle_funding_created_msg(self, msg: FundingCreated) -> (ChannelState, Option<Message>) {
        let local_htlc_pubkey = self.their_info.htlc_pubkey();
        let remote_htlc_pubkey = self.our_info.htlc_pubkey();

        let local_revocation_pubkey = self.their_info.revocation_pubkey(&self.our_info.revocation_bp_pk);

        let local_delayed_pubkey = self.their_info.delayed_pubkey();

        let remote_pubkey = self.our_info.payment_pubkey();

        let obscuring_factor = get_obscuring_number(
            &self.their_info.payment_bp_pk.serialize(),
            &self.our_info.payment_bp_pk.serialize()
        );

        let commit_tx = CommitTx{
            funding_amount: self.funding.funding as i64,
            local_funding_pubkey: self.their_info.funding_pk.clone(),
            remote_funding_pubkey: self.our_info.funding_pk.clone(),

            local_feerate_per_kw: u32::from(self.their_info.config.local_fee_rate) as i64,
            dust_limit_satoshi: u64::from(self.their_info.config.dust_limit) as i64,

            to_local_msat: (1000 * self.funding.funding) as i64,
            to_remote_msat: self.funding.push as i64,

            obscured_commit_number: 0 ^ obscuring_factor,

            local_htlc_pubkey: local_htlc_pubkey,
            remote_htlc_pubkey: remote_htlc_pubkey,

            local_revocation_pubkey: local_revocation_pubkey,
            local_delayedpubkey: local_delayed_pubkey,
            local_delay: self.their_info.config.csv_delay.into(),

            remotepubkey: remote_pubkey,

            funding_tx_id: Sha256dHash::from(&<[u8; 32]>::from(msg.funding_txid.clone())[..]),
            funding_output_index: u16::from(msg.output_index) as u32,

            htlcs: vec![]
        };
        let sig = commit_tx.sign(&self.our_info.funding_sk.unwrap());
        let tx = commit_tx.get_tx();
        let mut a = vec![];
        tx.consensus_encode(&mut RawEncoder::new(&mut a)).unwrap();

        let mut channel_id = <[u8; 32]>::from(msg.funding_txid);
        let ind = u16::from(msg.output_index);
        channel_id[0] ^= (ind & 0xFF) as u8;
        channel_id[1] ^= (ind >> 8) as u8;
        let funding_signed = FundingSigned {
            channel_id: ChannelId::from(channel_id),
            signature: LpdSignature::from(sig),
        };
        let data = WaitFundingLockedData {
            temp_channel_id: self.temp_channel_id,
            channel_id: channel_id.into(),
            funding: self.funding,
            our_info: self.our_info,
            obscuring_factor,
            their_info: self.their_info,
            funding_tx_id: Sha256dHash::from(&<[u8; 32]>::from(msg.funding_txid.clone())[..]),
            funding_output_index: msg.output_index.into(),
        };
        (
            ChannelState::Opening(OpeningState::WaitFundingLocked(data)),
            Some(Message::FundingSigned(funding_signed))
        )
    }
}

impl WaitFundingLockedData {
    fn handle_funding_locked_msg(self, msg: FundingLocked) -> (ChannelState, Option<Message>) {
        // TODO(mkl): check blockchain
        let funding_locked = FundingLocked {
            channel_id: msg.channel_id,
            next_per_commitment_point: LpdPublicKey::from(msg.next_per_commitment_point),
        };
        let data = ReadyState {
            channel_id: self.channel_id,
            our_info: self.our_info,
            their_info: self.their_info,
            obscuring_factor: self.obscuring_factor,
            funding_tx_id: self.funding_tx_id,
            funding_output_index: self.funding_output_index,
        };
        (
            ChannelState::Ready(data),
            Some(Message::FundingLocked(funding_locked))
        )
    }
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

    let mut your_commit_tx : Option<CommitTx> = None;
    let mut your_add_htlc : Option<UpdateAddHtlc> = None;
    let mut your_funding_locked : Option<FundingLocked> = None;
    let mut channel: Option<Channel> = None;

    let mut funding_info: FundingInfo = Default::default();

    let (fl_commit_point_sk, fl_commit_point_pk) = get_key_pair();

    let mut obscuring_factor : u64 = 0;
    let mut st = ChannelState::new();
    while true {
        match read_msg(&mut brontide_stream) {
            // Ping -> Pong
            Ok(Message::Ping(p)) =>  {
                println!("PING: {:?}", &p);
                let pong = Pong::new(&p);
                write_msg(&mut brontide_stream, &Message::Pong(pong)).unwrap();
            },

            // OpenChannel -> AcceptChannel OR Error
            Ok(Message::OpenChannel(open_channel)) => {
                println!("OPEN_CHANNEL: {:?}", open_channel);
                println!("chain_hash: {:?}", open_channel.chain_hash);
                let (new_st, resp) = st.next(Message::OpenChannel(open_channel));
                st = new_st;
                if resp.is_some() {
                    write_msg(&mut brontide_stream, &resp.unwrap()).unwrap();
                }
            },

            // FundingCreated -> FundingSigned OR ERROR
            Ok(Message::FundingCreated(funding_created)) => {
                println!("FUNDING_CREATED: {:?}", &funding_created);
                // Now we create a commitment transaction
                // local => from OpenChannel
                // remote => from AcceptChannel

                let (new_st, resp) = st.next(Message::FundingCreated(funding_created));
                st = new_st;
                if resp.is_some() {
                    write_msg(&mut brontide_stream, &resp.unwrap()).unwrap();
                }
            },

            // FundingLocked -> <channel opened>
            Ok(Message::FundingLocked(funding_locked)) => {
                println!("FUNDING_LOCKED: {:?}", &funding_locked);

                let (new_st, resp) = st.next(Message::FundingLocked(funding_locked));
                st = new_st;
                if resp.is_some() {
                    write_msg(&mut brontide_stream, &resp.unwrap()).unwrap();
                }
                println!(
                    "sendpayment --dest {} --amt {} --payment_hash {} --final_cltv_delta={}",
                    hex::encode(&local_pk.serialize()[..]),
                    1,
                    hex::encode(&rhash),
                    1,
                );
            },
            Ok(msg) => println!("MSG: {:?}", msg),
            Err(err) =>  {
//                println!("ERROR: {:?}", err)
            }
        }
    }

}
