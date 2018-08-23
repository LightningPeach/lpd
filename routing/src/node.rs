use bitcoin_types::ChannelPoint;
use common_types::Hash256;

use secp256k1::PublicKey;
use std::net::SocketAddr;

use wire::Color;
use wire::NodeAlias;
use wire::RawFeatureVector;
use wire::AnnouncementNodeData;
use wire::AnnouncementNode;
use wire::ShortChannelId;
use wire::PublicKey;
use wire::RawFeatureVector;
use wire::Satoshi;
use wire::Signature;

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
