use std::error::Error;

use super::ChannelId;
use super::OutputIndex;
use super::super::types::{RawSignature, RawPublicKey};

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub struct FundingTxid {
    data: [u8; 32],
}

/// This message describes the outpoint which the funder has created
/// for the initial commitment transactions. After receiving the peer's signature,
/// via `funding_signed`, it will broadcast the funding transaction.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct FundingCreated {
    /// the same as the `temporary_channel_id` in the open_channel message
    pub temporary_channel_id: ChannelId,
    /// the transaction ID of a non-malleable transaction
    pub funding_txid: FundingTxid,
    /// the output number of that transaction that corresponds
    /// the funding transaction output, as defined in BOLT #3
    pub output_index: OutputIndex,
    /// the valid signature using its funding_pubkey for
    /// the initial commitment transaction, as defined in BOLT #3
    pub signature: RawSignature,
}

/// This message gives the funder the signature it needs for the first commitment transaction,
/// so it can broadcast the transaction knowing that funds can be redeemed, if need be.
/// This message introduces the `channel_id` to identify the channel.
/// It's derived from the funding transaction by combining the `funding_txid` and
/// the `funding_output_index`, using big-endian exclusive-OR
/// (i.e. `funding_output_index` alters the last 2 bytes).
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct FundingSigned {
    pub channel_id: ChannelId,
    /// the valid signature, using its funding_pubkey for
    /// the initial commitment transaction, as defined in BOLT #3
    pub signature: RawSignature,
}

/// This message indicates that the funding transaction has reached the `minimum_depth`
/// asked for in `accept_channel`. Once both nodes have sent this,
/// the channel enters normal operating mode.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct FundingLocked {
    pub channel_id: ChannelId,
    pub next_per_commitment_point: RawPublicKey,
}

impl From<FundingTxid> for [u8; 32] {
    fn from(tx_id: FundingTxid) -> Self {
        return tx_id.data;
    }
}

impl std::fmt::Debug for FundingTxid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FundingTxid({})", self.to_hex())
    }
}

impl FundingTxid {
    // Use `reversed` byte ordering as Bitcoin uses when displaying transaction hashes
    pub fn to_hex(&self) -> String {
        let mut b: [u8; 32] = [0; 32];
        for i in 0..32 {
            b[i] = self.data[31-i];
        }
        hex::encode(&b[..])
    }

    pub fn to_hex_normal(&self) -> String {
        hex::encode(&self.data[..])
    }

    pub fn from_hex(s: &str) -> Result<FundingTxid, Box<Error>> {
        let data = hex::decode(s)
            .map_err(|err| format!("cannot decode from hex FundingTxid: {:?}", err))?;
        if data.len() != 32 {
            return Err(format!("incorrect byte length of FundingTxid, got {}, want {}", data.len(), 32).into());
        }

        // string representation is reverse ordered
        let mut data_u8_32: [u8; 32] = [0; 32];
        for i in 0..32 {
            data_u8_32[i] = data[31-i];
        }

        Ok(FundingTxid {
            data: data_u8_32
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use binformat::BinarySD;
    use crate::message::channel::{ChannelId, OutputIndex};
    use crate::message::{FundingCreated, FundingSigned};
    use std::io::Cursor;
    use crate::Message;
    use pretty_assertions::assert_eq;

    #[test]
    fn funding_created_test() {
        let msg_hex = "002202000000000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000200050000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = FundingCreated {
            temporary_channel_id: ChannelId::from_hex("0200000000000000000000000000000000000000000000000000000000000000").unwrap(),
            funding_txid: FundingTxid::from_hex("0000000000000000000000000000000000000000000000000000000005000000").unwrap(),
            output_index: OutputIndex::from_u16(2),
            signature: RawSignature::from_hex("3024021f05000000000000000000000000000000000000000000000000000000000000020100").unwrap(),
        };
        let wrapped_msg_correct = Message::FundingCreated(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }

    #[test]
    fn funding_signed_test() {
        let msg_hex = "0023000500000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = FundingSigned {
            channel_id: ChannelId::from_hex("0005000000000000000000000000000000000000000000000000000000000000").unwrap(),
            signature: RawSignature::from_hex("3024021f03000000000000000000000000000000000000000000000000000000000000020100").unwrap(),
        };
        let wrapped_msg_correct = Message::FundingSigned(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }

    #[test]
    fn funding_locked_test() {
        let msg_hex = "002401000000000000000000000000000000000000000000000000000000000000000227efef5cab173f127b0a50bf1839c61287e440669e44fe8cf9cf6ef7bcd9ee00";
        let msg_bytes = hex::decode(msg_hex).unwrap();
        let msg_correct = FundingLocked{
            channel_id: ChannelId::from_hex("0100000000000000000000000000000000000000000000000000000000000000").unwrap(),
            next_per_commitment_point: RawPublicKey::from_hex("0227efef5cab173f127b0a50bf1839c61287e440669e44fe8cf9cf6ef7bcd9ee00").unwrap(),
        };
        let wrapped_msg_correct = Message::FundingLocked(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }
}