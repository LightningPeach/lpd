use super::Signature;
use super::RawFeatureVector;
use super::Hash256;
use super::PublicKey;
use super::ShortChannelId;

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
#[allow(unused_imports)]
mod test {
    use super::*;
    use ::BinarySD;

    #[test]
    fn announcement_channel() {
        // TODO:
    }
}
