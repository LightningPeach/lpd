use super::Signed;
use super::RawFeatureVector;
use super::PublicKey;
use super::NodeAlias;
use super::Color;

pub type AnnouncementNode = Signed<AnnouncementNodeData>;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct AnnouncementNodeData {
    features: RawFeatureVector,
    timestamp: u32,
    node_id: PublicKey,
    color: Color,
    alias: NodeAlias,
    address: Vec<u8>,
}

// TODO:
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum Address {
    None,
    IpV4,
    IpV6,
    TorV2,
    TorV3,
}
