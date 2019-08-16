use dependencies::secp256k1;
use dependencies::bitcoin_hashes;
use dependencies::bitcoin;
use dependencies::rand;


use wire::{
    Message, AcceptChannel, OpenChannel,
    FundingSigned, ChannelId, FundingLocked, Satoshi, MilliSatoshi, CsvDelay, FundingCreated,
    ChannelKeys, ChannelPrivateKeys, RawSignature,
};

use secp256k1::PublicKey;

use bitcoin_hashes::{sha256d, sha256};
use bitcoin_hashes::Hash;

use bitcoin::consensus::encode::Encodable;

use channel::derivation::{derive_pubkey, derive_revocation_pubkey};
use channel::tools::get_obscuring_number;
use channel::commit::CommitTx;

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

#[derive(Debug, Clone)]
struct PartnerInfo {
    keys: ChannelKeys,
    private_keys: Option<ChannelPrivateKeys>,
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
        let private_keys: ChannelPrivateKeys = rand::random();
        let keys = ChannelKeys::new(&private_keys);

        PartnerInfo {
            keys: keys,
            private_keys: Some(private_keys),
            config: Default::default(),
        }
    }

    fn from_open_channel_msg(msg: &OpenChannel) -> PartnerInfo {
        let config = PartnerConfig {
            dust_limit: u64::from(msg.dust_limit),
            max_htlc_value_in_flight: u64::from(msg.max_in_flight),
            chanel_reserve: u64::from(msg.channel_reserve),
            htlc_minimum: u64::from(msg.htlc_minimum),
            csv_delay: u16::from(msg.csv_delay),
            max_accepted_htlc_number: u16::from(msg.max_accepted_htlc_number),
            local_fee_rate: u32::from(msg.fee),
        };

        PartnerInfo {
            keys: msg.keys.clone(),
            private_keys: None,
            config,
        }
    }

    fn htlc_pubkey(&self) -> PublicKey {
        derive_pubkey(&self.keys.htlc(), &self.keys.first_per_commitment())
    }

    // Revocation pubkey for our commitment transaction
    // We require partners revocation_pk to generate it
    // and our commitment point
    fn revocation_pubkey(&self, partner_revocation_pk: &PublicKey) -> PublicKey {
        derive_revocation_pubkey(partner_revocation_pk, &self.keys.first_per_commitment())
    }

    // We use this pubkey to send money to ourself
    fn delayed_pubkey(&self) -> PublicKey {
        derive_pubkey(&self.keys.delayed_payment(), &self.keys.first_per_commitment())
    }

    fn payment_pubkey(&self) -> PublicKey {
        derive_pubkey(&self.keys.payment(), &self.keys.first_per_commitment())
    }
}

#[derive(Debug, Clone)]
pub struct InitialState;

// TODO(mkl): refactor into enum
#[derive(Debug, Clone)]
pub struct ReadyState {
    channel_id: ChannelId,
    our_info: PartnerInfo,
    their_info: PartnerInfo,
    obscuring_factor: u64,
    funding_tx_id: sha256d::Hash,
    funding_output_index: u16,
}

#[derive(Debug, Clone)]
pub enum ChannelState {

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
pub struct WaitFundingCreatedData {
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
pub struct WaitFundingLockedData {
    temp_channel_id: ChannelId,
    channel_id: ChannelId,
    our_info: PartnerInfo,
    their_info: PartnerInfo,
    obscuring_factor: u64,
    funding: FundingInfo,
    funding_tx_id: sha256d::Hash,
    funding_output_index: u16,
}

#[derive(Debug, Clone)]
pub enum OpeningState {
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
    pub fn new() -> ChannelState {
        ChannelState::Initial(InitialState)
    }

    pub fn next(self, msg: Message) -> (ChannelState, Option<Message>) {
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
        let accept_channel_msg = AcceptChannel {
            temporary_channel_id: msg.temporary_channel_id.clone(),
            dust_limit: Satoshi::from(their_info.config.dust_limit),
            max_htlc_value_in_flight: MilliSatoshi::from(their_info.config.max_htlc_value_in_flight),
            chanel_reserve: Satoshi::from(their_info.config.chanel_reserve),
            htlc_minimum: MilliSatoshi::from(their_info.config.htlc_minimum),
            minimum_accept_depth: 1,
            csv_delay: CsvDelay::from(their_info.config.csv_delay),
            max_accepted_htlc_number: their_info.config.max_accepted_htlc_number,
            keys: our_info.keys.clone(),
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

        let local_revocation_pubkey = self.their_info.revocation_pubkey(&self.our_info.keys.revocation());

        let local_delayed_pubkey = self.their_info.delayed_pubkey();

        let remote_pubkey = self.our_info.payment_pubkey();

        let obscuring_factor = get_obscuring_number(
            &self.their_info.keys.payment().serialize(),
            &self.our_info.keys.payment().serialize()
        );

        let commit_tx = CommitTx {
            funding_amount: self.funding.funding as i64,
            local_funding_pubkey: self.their_info.keys.funding().clone(),
            remote_funding_pubkey: self.our_info.keys.funding().clone(),

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

            funding_tx_id: msg.funding_txid.to_sha256d(),
            funding_output_index: u16::from(msg.output_index) as u32,

            htlcs: vec![],
        };
        let sig = commit_tx.sign(self.our_info.private_keys.clone().unwrap().funding_sk());
        let tx = commit_tx.get_tx();
        let mut a = vec![];
        tx.consensus_encode(&mut a).unwrap();

        let mut channel_id = <[u8; 32]>::from(msg.funding_txid);
        let ind = u16::from(msg.output_index);
        channel_id[0] ^= (ind & 0xFF) as u8;
        channel_id[1] ^= (ind >> 8) as u8;
        let funding_signed = FundingSigned {
            channel_id: ChannelId::from(channel_id),
            signature: RawSignature(sig),
        };
        let data = WaitFundingLockedData {
            temp_channel_id: self.temp_channel_id,
            channel_id: channel_id.into(),
            funding: self.funding,
            our_info: self.our_info,
            obscuring_factor,
            their_info: self.their_info,
            funding_tx_id: msg.funding_txid.to_sha256d(),
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
            next_per_commitment_point: msg.next_per_commitment_point,
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
