use bitcoin_types::ChannelPoint;
use common_types::Hash256;

use chrono::DateTime;
use chrono::Utc;

use std::net::SocketAddr;

use wire::Color;
use wire::NodeAlias;
use wire::RawFeatureVector;
use wire::AnnouncementNodeData;
use wire::AnnouncementNode;
use wire::ShortChannelId;
use wire::PublicKey;
use wire::Satoshi;
use wire::Signature;
use wire::Signed;
use wire::ChannelUpdateFlags;
use wire::MilliSatoshi;

pub struct LightningNode {
    public_key: PublicKey,
    have_node_announcement: bool,
    last_update: u32,
    address: SocketAddr,
    color: Color,
    alias: NodeAlias,
    auth_signature_key: PublicKey,
    features: RawFeatureVector,
    // TODO(vlad):
    db: (),
}

pub struct ChannelAuthProof {
    node_id: (Signature, Signature),
    bitcoin_key: (Signature, Signature),
}

pub struct ChannelInfo {
    channel_id: ShortChannelId,
    chain_hash: Hash256,
    node_id: (PublicKey, PublicKey),
    bitcoin_key: (PublicKey, PublicKey),
    features: RawFeatureVector,
    auth_proof: ChannelAuthProof,
    channel_point: ChannelPoint,
    capacity: Satoshi,
    // TODO(vlad):
    db: (),
}

pub type ChannelPolicy = Signed<ChannelPolicyData>;

// TODO: the struct is similar to wire::UpdateChannelData, reuse it somehow
pub struct ChannelPolicyData {
    short_channel_id: ShortChannelId,
    last_update_time: DateTime<Utc>,
    flags: ChannelUpdateFlags,
    time_lock_delta: u16,
    htlc_minimum: MilliSatoshi,
    base_fee: u32,
    fee_rate: u32,
    node: LightningNode,
    // TODO(vlad):
    db: (),
}
