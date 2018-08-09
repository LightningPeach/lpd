use bitcoin;
use hex;

use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
use bitcoin::util::hash::{Sha256dHash, Hash160, Ripemd160Hash};
use bitcoin::blockdata::script::{Script, Builder};
use bitcoin::blockdata::opcodes::All::*;

use bitcoin::network::encodable::{ConsensusEncodable, ConsensusDecodable};
use bitcoin::network::serialize::{RawEncoder, RawDecoder};

use bitcoin::util::bip143;
use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};

use crypto::sha2::Sha256;
use crypto::digest::Digest;

pub const OP_CHECKSEQUENCEVERIFY: bitcoin::blockdata::opcodes::All = OP_NOP3;
pub const OP_CHECKLOCKTIMEVERIFY: bitcoin::blockdata::opcodes::All = OP_NOP2;

pub fn s2dh256(s: &str) -> Sha256dHash {
    match Sha256dHash::from_hex(s) {
        Ok(h) => return h,
        Err(e) => panic!(e)
    }
}

pub fn s2script(s: &str) -> Script {
    let b = match hex::decode(s) {
        Ok(r) => r,
        Err(e) => panic!(e)
    };
    let sc = Script::from(b);
    return sc;
}

pub fn s2byte32(s: &str) -> [u8; 32]{
    let data = hex::decode(s).unwrap();
    if data.len() != 32 {
        panic!("incorrect length of data");
    }
    let mut rez: [u8; 32] = [0; 32];
    for i in 0..32 {
        rez[i] = data[i];
    }
    return rez;
}

// TODO(mkl): check if ordering is correct (from spec about pubkey ordering)
pub fn ordered<'a>(pk1: &'a[u8], pk2: &'a[u8]) -> (&'a[u8], &'a[u8]) {
    if pk1 < pk2 {
        return (pk1, pk2);
    } else {
        return (pk2, pk1);
    }
}

pub fn new_2x2_multisig(pk1: &[u8], pk2: &[u8]) -> Script {
    let (pk1, pk2) = ordered(pk1, pk2);
    let b = Builder::new();
    let b = b
        .push_opcode(OP_PUSHNUM_2)
        .push_slice(pk1)
        .push_slice(pk2)
        .push_opcode(OP_PUSHNUM_2)
        .push_opcode(OP_CHECKMULTISIG);
    return b.into_script();
}

pub fn new_2x2_wsh_lock_script(pk1: &[u8], pk2: &[u8]) -> Script {
    let sc = new_2x2_multisig(pk1, pk2);
    return sc.to_v0_p2wsh()
}

pub fn v0_p2wpkh(pk: &PublicKey) -> Script {
    let pk_hash160 = Hash160::from_data(&pk.serialize()[..]).data();
    let sc = Builder::new()
        .push_opcode(OP_PUSHBYTES_0)
        .push_slice(&pk_hash160)
        .into_script();
    return sc;
}

pub fn p2pkh(pk: &PublicKey) -> Script {
    // OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG
    let pk_hash160 = Hash160::from_data(&pk.serialize()[..]).data();
    let sc = Builder::new()
        .push_opcode(OP_DUP)
        .push_opcode(OP_HASH160)
        .push_slice(&pk_hash160)
        .push_opcode(OP_EQUALVERIFY)
        .push_opcode(OP_CHECKSIG)
        .into_script();
    return sc;
}

pub fn p2pkh_unlock_script(pk: &PublicKey, sig: &[u8]) -> Script {
    let pk_serialised = pk.serialize();
    let sc = Builder::new()
        .push_slice(sig)
        .push_slice(&pk_serialised)
        .into_script();
    return sc;
}

pub fn s2privkey(s: &str) -> SecretKey {
    let b = hex::decode(s).unwrap();
    if !(b.len() == 32 || (b.len()==33 && b[32]==1)) {
        panic!("incorrect string size");
    }
    let k = SecretKey::from_slice(&Secp256k1::new(), &b[0..32]).unwrap();
    return k;
}

pub fn s2pubkey(s: &str) -> PublicKey {
    let b = hex::decode(s).unwrap();
    let pk = PublicKey::from_slice(&Secp256k1::new(), &b).unwrap();
    return pk;
}