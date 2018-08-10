use super::Signature;
use super::RawFeatureVector;
use super::Hash256;
use super::PublicKey;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct ShortChannelId {
    block_height: u32,
    tx_index: u32,
    tx_position: u16,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct AnnouncementChannel {
    node_sig: (Signature, Signature),
    bitcoin_sig: (Signature, Signature),
    features: RawFeatureVector,
    chain_hash: Hash256,
    short_channel_id: ShortChannelId,
    node_id: (PublicKey, PublicKey),
    bitcoin_key: (PublicKey, PublicKey),
}

#[cfg(test)]
mod test {
    use super::*;
    use ::BinarySD;

    #[test]
    fn announcement_channel() {
        // TODO:
    }
}
