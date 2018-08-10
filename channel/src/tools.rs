use bitcoin;
use hex;

use bitcoin::blockdata::transaction::{Transaction};
use bitcoin::util::hash::{Sha256dHash, Hash160, Ripemd160Hash};
use bitcoin::blockdata::script::{Script, Builder};
use bitcoin::blockdata::opcodes::All::*;

use bitcoin::network::encodable::{ConsensusDecodable};
use bitcoin::network::serialize::{RawDecoder};

use secp256k1::{Secp256k1, SecretKey, PublicKey};

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

pub fn s2tx(s: &str) -> Transaction {
    let tx_bytes = hex::decode(s).unwrap();
    let tx = Transaction::consensus_decode(&mut RawDecoder::new(&tx_bytes[..])).unwrap();
    return tx;
}

// Get sequence number from obscured commit number
// upper 8 bits are 0x80,
// lower 24 bits are upper 24 bits of the obscured commitment transaction number
pub fn get_sequence(x: u64) -> u64 {
    return (0x80 << 24) + (x >> 24);
}

// Get sequence number from obscured sequence number
// upper 8 bits are 0x20,
// lower 24 bits are the lower 24 bits of the obscured commitment transaction number
pub fn get_locktime(x: u64) -> u64 {
    return (0x20 << 24) + (x & 0xFFFFFF);
}


//OP_IF
//    # Penalty transaction
//    <revocationpubkey>
//OP_ELSE
//    `to_self_delay`
//    OP_CSV
//    OP_DROP
//    <local_delayedpubkey>
//OP_ENDIF
//OP_CHECKSIG
pub fn to_local_script(local_delayedpubkey: &PublicKey, to_self_delay: u64, revocationpubkey: &PublicKey) -> Script {
    let sc = Builder::new()
        .push_opcode(OP_IF)
        .push_slice(&revocationpubkey.serialize())
        .push_opcode(OP_ELSE)
        .push_int(to_self_delay as i64)
        .push_opcode(OP_CHECKSEQUENCEVERIFY)
        .push_opcode(OP_DROP)
        .push_slice(&local_delayedpubkey.serialize())
        .push_opcode(OP_ENDIF)
        .push_opcode(OP_CHECKSIG)
        .into_script();
    return sc;
}

pub fn get_obscuring_number(local_payment_basepoint: &[u8], remote_payment_basepoint: &[u8]) -> u64 {

    let concated = [local_payment_basepoint, remote_payment_basepoint].concat();

    let mut rez = [0_u8; 32];
    let mut sh = Sha256::new();
    sh.input(&concated);
    sh.result(&mut rez[..]);

    let mut obscuring_number = 0;
    for i in 0..6 {
        obscuring_number += (rez[31-i] as u64) << (i*8);
    }
    return obscuring_number;
}

pub fn sha256(x: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.input(x);
    let mut hash: [u8; 32] = [0; 32];
    h.result(&mut hash);
    return hash;
}

//OP_DUP OP_HASH160 <RIPEMD160(SHA256(revocationpubkey))> OP_EQUAL
//OP_IF
//    OP_CHECKSIG
//OP_ELSE
//    <remote_htlcpubkey> OP_SWAP OP_SIZE 32 OP_EQUAL
//    OP_NOTIF
//        # To local node via HTLC-timeout transaction (timelocked).
//        OP_DROP 2 OP_SWAP <local_htlcpubkey> 2 OP_CHECKMULTISIG
//    OP_ELSE
//        # To remote node with preimage.
//        OP_HASH160 <RIPEMD160(payment_hash)> OP_EQUALVERIFY
//        OP_CHECKSIG
//    OP_ENDIF
//OP_ENDIF
pub fn offered_htlc(revocationpubkey: &PublicKey, remote_htlcpubkey: &PublicKey, local_htlcpubkey: &PublicKey, payment_hash: [u8; 32]) -> Script {
    let sc = Builder::new()
        .push_opcode(OP_DUP)
        .push_opcode(OP_HASH160)
        .push_slice(&Hash160::from_data(&revocationpubkey.serialize()).data())
        .push_opcode(OP_EQUAL)
        .push_opcode(OP_IF)
            .push_opcode(OP_CHECKSIG)
        .push_opcode(OP_ELSE)
            .push_slice(&remote_htlcpubkey.serialize())
            .push_opcode(OP_SWAP)
            .push_opcode(OP_SIZE)
            .push_int(32)
            .push_opcode(OP_EQUAL)
            .push_opcode(OP_NOTIF)
                //OP_DROP 2 OP_SWAP <local_htlcpubkey> 2 OP_CHECKMULTISIG
                .push_opcode(OP_DROP)
                .push_int(2)
                .push_opcode(OP_SWAP)
                .push_slice(&local_htlcpubkey.serialize())
                .push_int(2)
                .push_opcode(OP_CHECKMULTISIG)
            .push_opcode(OP_ELSE)
                // To remote node with preimage.
                // OP_HASH160 <RIPEMD160(payment_hash)> OP_EQUALVERIFY
                // OP_CHECKSIG
                .push_opcode(OP_HASH160)
                .push_slice(&Ripemd160Hash::from_data(&payment_hash).data())
                .push_opcode(OP_EQUALVERIFY)
                .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_ENDIF)
        .push_opcode(OP_ENDIF)
        .into_script();
    return sc;
}

//# To remote node with revocation key
//OP_DUP OP_HASH160 <RIPEMD160(SHA256(revocationpubkey))> OP_EQUAL
//OP_IF
//    OP_CHECKSIG
//OP_ELSE
//    <remote_htlcpubkey> OP_SWAP
//        OP_SIZE 32 OP_EQUAL
//    OP_IF
//        # To local node via HTLC-success transaction.
//        OP_HASH160 <RIPEMD160(payment_hash)> OP_EQUALVERIFY
//        2 OP_SWAP <local_htlcpubkey> 2 OP_CHECKMULTISIG
//    OP_ELSE
//        # To remote node after timeout.
//        OP_DROP <cltv_expiry> OP_CHECKLOCKTIMEVERIFY OP_DROP
//        OP_CHECKSIG
//    OP_ENDIF
//OP_ENDIF
pub fn accepted_htlc(revocationpubkey: &PublicKey, remote_htlcpubkey: &PublicKey, local_htlcpubkey: &PublicKey, payment_hash: [u8; 32], cltv_expiry: u32) -> Script {
    let sc = Builder::new()
        .push_opcode(OP_DUP)
        .push_opcode(OP_HASH160)
        .push_slice(&Hash160::from_data(&revocationpubkey.serialize()).data())
        .push_opcode(OP_EQUAL)
        .push_opcode(OP_IF)
            .push_opcode(OP_CHECKSIG)
        .push_opcode(OP_ELSE)
            .push_slice(&remote_htlcpubkey.serialize())
            .push_opcode(OP_SWAP)
            .push_opcode(OP_SIZE)
            .push_int(32)
            .push_opcode(OP_EQUAL)
            .push_opcode(OP_IF)
                // # To local node via HTLC-success transaction.
                .push_opcode(OP_HASH160)
                .push_slice(&Ripemd160Hash::from_data(&payment_hash).data())
                .push_opcode(OP_EQUALVERIFY)
                .push_int(2)
                .push_opcode(OP_SWAP)
                .push_slice(&local_htlcpubkey.serialize())
                .push_int(2)
                .push_opcode(OP_CHECKMULTISIG)
            .push_opcode(OP_ELSE)
                // # To remote node after timeout.
                .push_opcode(OP_DROP)
                .push_int(cltv_expiry as i64)
                .push_opcode(OP_CHECKLOCKTIMEVERIFY)
                .push_opcode(OP_DROP)
                .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_ENDIF)
        .push_opcode(OP_ENDIF)
        .into_script();
    return sc;
}


pub fn assert_tx_eq(tx1: &Transaction, tx2: &Transaction, ignore_witness: bool) {
    assert_eq!(tx1.version, tx2.version);
    assert_eq!(tx1.input.len(), tx2.input.len());
    for i in 0..tx1.input.len() {
        assert_eq!(tx1.input[i].prev_hash, tx2.input[i].prev_hash);
        assert_eq!(tx1.input[i].prev_index, tx2.input[i].prev_index);
        assert_eq!(tx1.input[i].script_sig, tx2.input[i].script_sig);
        assert_eq!(tx1.input[i].sequence, tx2.input[i].sequence);
        if !ignore_witness {
            assert_eq!(tx1.input[i].witness.len(), tx2.input[i].witness.len());
            for j in 0..tx1.input[i].witness.len() {
                assert_eq!(tx1.input[i].witness[j], tx2.input[i].witness[j]);
            }
        }
    }
    assert_eq!(tx1.output.len(), tx2.output.len());
    for i in 0..tx1.output.len() {
        assert_eq!(tx1.output[i].value, tx2.output[i].value);
        assert_eq!(tx1.output[i].script_pubkey, tx2.output[i].script_pubkey);
    }
    assert_eq!(tx1.lock_time, tx2.lock_time);
}

