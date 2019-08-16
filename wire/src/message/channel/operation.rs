use dependencies::hex;

use super::ChannelId;
use super::Hash256;
use super::MilliSatoshi;
use super::OnionBlob;
use super::SatoshiPerKiloWeight;
use super::super::types::{RawSignature, RawPublicKey};

use serde_derive::{Serialize, Deserialize};

use std::error::Error;

pub fn u8_32_from_hex(s: &str) -> Result<[u8; 32], Box<Error>> {
    let bytes = hex::decode(s.as_bytes())
        .map_err(|err| format!("cannot decode hex: {:?}", err))?;
    if bytes.len() != 32 {
        return Err(format!("incorrect byte length, got {}, want {}", bytes.len(), 32).into());
    }
    let mut data = [0; 32];
    data.copy_from_slice(&bytes);
    Ok(data)
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
pub struct HtlcId {
    id: u64,
}

impl HtlcId {
    pub fn new() -> Self {
        HtlcId {
            id: 0,
        }
    }

    pub fn next(&self) -> Self {
        HtlcId {
            id: self.id + 1,
        }
    }

    pub fn to_u64(&self) -> u64 {
        self.id
    }

    pub fn from_u64(x: u64) ->Self {
        HtlcId{
            id: x
        }
    }
}

/// Either node can send `update_add_htlc` to offer an HTLC to the other,
/// which is redeemable in return for a payment preimage. Amounts are in millisatoshi,
/// though on-chain enforcement is only possible for whole satoshi amounts
/// greater than the dust limit (in commitment transactions these are rounded down
/// as specified in BOLT #3).
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct UpdateAddHtlc {
    pub channel_id: ChannelId,
    pub id: HtlcId,
    pub amount: MilliSatoshi,
    pub payment_hash: Hash256,
    pub expiry: u32,
    pub onion_blob: OnionBlob,
}

/// Remove HTLC if the payment preimage is supplied.
// TODO(mkl): maybe add types PaymentHash, PaymentPreImage
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct UpdateFulfillHtlc {
    pub channel_id: ChannelId,
    pub id: HtlcId,
    pub payment_preimage: Hash256,
}

/// Remove HTLC if it has timed out or it has failed to route.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct UpdateFailHtlc {
    pub channel_id: ChannelId,
    pub id: HtlcId,
    pub reason: Vec<u8>,
}

/// Remove HTLC if it is malformed.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct UpdateFailMalformedHtlc {
    pub channel_id: ChannelId,
    pub id: HtlcId,
    pub sha256_of_onion: Hash256,
    pub failure_code: u16,
}

/// When a node has changes for the remote commitment, it can apply them,
/// sign the resulting transaction (as defined in BOLT #3), and send a `commitment_signed` message.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct CommitmentSigned {
    pub channel_id: ChannelId,
    pub signature: RawSignature,
    pub htlc_signatures: Vec<RawSignature>,
}

/// Once the recipient of `commitment_signed` checks the signature
/// and knows it has a valid new commitment transaction,
/// it replies with the commitment preimage for the previous commitment transaction
/// in a `revoke_and_ack` message.
/// This message also implicitly serves as an acknowledgment
/// of receipt of the `commitment_signed`, so this is a logical time
/// for the `commitment_signed` sender to apply (to its own commitment)
/// any pending updates it sent before that `commitment_signed`.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct RevokeAndAck {
    pub channel_id: ChannelId,
    pub revocation_preimage: Hash256,
    pub next_per_commitment_point: RawPublicKey,
}

/// An `update_fee` message is sent by the node which is paying the Bitcoin fee.
/// Like any update, it's first committed to the receiver's commitment transaction
/// and then (once acknowledged) committed to the sender's. Unlike an HTLC,
/// `update_fee` is never closed but simply replaced.
/// There is a possibility of a race, as the recipient can add new HTLCs
/// before it receives the `update_fee`. Under this circumstance,
/// the sender may not be able to afford the fee on its own commitment transaction,
/// once the `update_fee` is finally acknowledged by the recipient.
/// In this case, the fee will be less than the fee rate, as described in BOLT #3.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct UpdateFee {
    pub channel_id: ChannelId,
    pub fee: SatoshiPerKiloWeight,
}

#[cfg(test)]
mod test {
    use dependencies::hex;
    use dependencies::pretty_assertions;

    use binformat::BinarySD;
    use crate::message::channel::ChannelId;
    use crate::message::channel::operation::{UpdateFulfillHtlc, HtlcId, u8_32_from_hex};
    use std::io::Cursor;
    use crate::{
        Message, RevokeAndAck, RawPublicKey, CommitmentSigned, RawSignature, UpdateAddHtlc,
        MilliSatoshi, OnionBlob, UpdateFailHtlc, UpdateFailMalformedHtlc, SatoshiPerKiloWeight
    };
    use pretty_assertions::assert_eq;
    use common_types::Hash256;
    use super::UpdateFee;

    #[test]
    fn update_fulfill_htlc_test() {
        let msg_hex = "\
            00820200000000000000000000000000000000000000000000000000000000000000000000\
            00000000790064000000000000000000000000000000000000000000000000000000000000";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = UpdateFulfillHtlc {
            channel_id: ChannelId::from_hex("0200000000000000000000000000000000000000000000000000000000000000").unwrap(),
            id: HtlcId::from_u64(121),
            payment_preimage: Hash256::from_hex("0064000000000000000000000000000000000000000000000000000000000000").unwrap()
        };
        let wrapped_msg_correct = Message::UpdateFulfillHtlc(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }


    #[test]
    fn revoke_and_ack_test() {
        let msg_hex = "\
            00850100000000000000000000000000000000000000000000000000000000000000000200000000\
            000000000000000000000000000000000000000000000000000002122fac0daa5028e984f52bf5fa\
            72cb0ec7bf3758fcb8392a6a2ef71a9d00d994";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = RevokeAndAck{
            channel_id: ChannelId::from_hex("0100000000000000000000000000000000000000000000000000000000000000").unwrap(),
            revocation_preimage: Hash256::from_hex("0002000000000000000000000000000000000000000000000000000000000000").unwrap(),
            next_per_commitment_point: RawPublicKey::from_hex("02122fac0daa5028e984f52bf5fa72cb0ec7bf3758fcb8392a6a2ef71a9d00d994").unwrap(),
        };
        let wrapped_msg_correct = Message::RevokeAndAck(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }


    #[test]
    fn commitment_signed_test() {
        let msg_hex = "\
            00840100000000000000000000000000000000000000000000000000000000000000000200000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000020003000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000040000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = CommitmentSigned {
            channel_id: ChannelId::from_hex("0100000000000000000000000000000000000000000000000000000000000000").unwrap(),
            signature: RawSignature::from_hex("3024021f02000000000000000000000000000000000000000000000000000000000000020100").unwrap(),
            htlc_signatures: vec![
                RawSignature::from_hex("3024021f03000000000000000000000000000000000000000000000000000000000000020100").unwrap(),
                RawSignature::from_hex("3022021d0400000000000000000000000000000000000000000000000000000000020100").unwrap(),
            ],
        };
        let wrapped_msg_correct = Message::CommitmentSigned(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }

    #[test]
    fn update_add_htlc_test() {
        let msg_hex = "\
            00800200000000000000000000000000000000000000000000000000000000000000000000000000\
            03e90000000000018a88007900000000000000000000000000000000000000000000000000000000\
            00000000006400050000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000c8";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = UpdateAddHtlc {
            channel_id: ChannelId::from_hex("0200000000000000000000000000000000000000000000000000000000000000").unwrap(),
            id: HtlcId::from_u64(1001),
            amount: MilliSatoshi::from(101000),
            payment_hash: Hash256::from_hex("0079000000000000000000000000000000000000000000000000000000000000").unwrap(),
            expiry: 100,
            onion_blob: OnionBlob::from_hex("\
                00050000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                00000000000000000000000000000000000000000000000000000000000000000000000000000000\
                0000000000c8\
            ").unwrap(),
        };
        let wrapped_msg_correct = Message::UpdateAddHtlc(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }


    #[test]
    fn update_fail_htlc_test() {
        let msg_hex = "00830400000000000000000000000000000000000000000000000000000000000000000000000000271a000401020105";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = UpdateFailHtlc {
            channel_id: ChannelId::from_hex("0400000000000000000000000000000000000000000000000000000000000000").unwrap(),
            id: HtlcId::from_u64(10010),
            reason: hex::decode("01020105").unwrap(),
        };
        let wrapped_msg_correct = Message::UpdateFailHtlc(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }


    #[test]
    fn update_fail_malformed_htlc_test() {
        let msg_hex = "\
            0087020000000000000000000000000000000000000000000000000000000000000000000000\
            000000640000000500000000000000000000000000000000000000000000000000000000c005";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = UpdateFailMalformedHtlc {
            channel_id: ChannelId::from_hex("0200000000000000000000000000000000000000000000000000000000000000").unwrap(),
            id: HtlcId::from_u64(100),
            sha256_of_onion: Hash256::from_hex("0000000500000000000000000000000000000000000000000000000000000000").unwrap(),
            failure_code: 49157,
        };
        let wrapped_msg_correct = Message::UpdateFailMalformedHtlc(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }

    #[test]
    fn update_fee_test() {
        let msg_hex = "00860200000000000000000000000000000000000000000000000000000000000000000003e9";
        let msg_bytes = hex::decode(msg_hex).unwrap();

        let msg_correct = UpdateFee {
            channel_id: ChannelId::from_hex("0200000000000000000000000000000000000000000000000000000000000000").unwrap(),
            fee: SatoshiPerKiloWeight::from(1001),
        };
        let wrapped_msg_correct = Message::UpdateFee(msg_correct);

        let mut cursor = Cursor::new(msg_bytes.clone());
        let msg = BinarySD::deserialize::<Message, _>(&mut cursor).unwrap();
        assert_eq!(&msg, &wrapped_msg_correct);

        // Now check deserialization
        let mut new_msg_bytes = vec![];
        BinarySD::serialize(&mut new_msg_bytes, &wrapped_msg_correct).unwrap();
        assert_eq!(new_msg_bytes, msg_bytes);
    }


}