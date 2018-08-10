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


#[cfg(test)]
mod tests {

    use hex;
    use tools::{sha256, accepted_htlc, offered_htlc, assert_tx_eq, to_local_script, s2script, s2tx, new_2x2_multisig, new_2x2_wsh_lock_script, s2pubkey, v0_p2wpkh, s2dh256, p2pkh, p2pkh_unlock_script, get_obscuring_number, get_locktime, get_sequence};
    use spec_example::get_example;
    use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
    use bitcoin::util::hash::Hash160;
    use bitcoin::blockdata::script::Script;
    use bitcoin::network::serialize::{RawEncoder};
    use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
    use bitcoin::network::encodable::ConsensusEncodable;
    use bitcoin::util::bip143;

    #[test]
    fn test_new_2x2_multisig() {
        // Try to recreate example from specification (funding transaction):
        let local_pk = hex::decode("023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb").unwrap();
        let remote_pk = hex::decode("030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c1").unwrap();
        let funding_ws = new_2x2_multisig(&local_pk, &remote_pk);
        assert_eq!(hex::encode(funding_ws.data()), "5221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae");
    }

    #[test]
    fn test_new_2x2_wsh_lock_script(){
        // Try to recreate example from specification:
        let local_pk = hex::decode("023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb").unwrap();
        let remote_pk = hex::decode("030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c1").unwrap();
        // Funding lockscript should be "0020c015c4a6be010e21657068fc2e6a9d02b27ebe4d490a25846f7237f104d1a3cd"
        let funding_lock_script = new_2x2_wsh_lock_script(&local_pk, &remote_pk);
        assert_eq!(hex::encode(funding_lock_script.data()), "0020c015c4a6be010e21657068fc2e6a9d02b27ebe4d490a25846f7237f104d1a3cd");
    }

    #[test]
    fn test_v0_p2wpkh() {
        let pk = s2pubkey("03535b32d5eb0a6ed0982a0479bbadc9868d9836f6ba94dd5a63be16d875069184");
        // Change output script should be 00143ca33c2e4446f4a305f23c80df8ad1afdcf652f9
        // it is change output of the funding transaction from the spec
        let sc_change = v0_p2wpkh(&pk);
        assert_eq!(hex::encode(&sc_change.data()), "00143ca33c2e4446f4a305f23c80df8ad1afdcf652f9");
    }

    #[test]
    fn test_funding_transaction() {
        // Recreates funding transaction from specification
        // Try to recreate example from specification:
        let local_pk = hex::decode("023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb").unwrap();
        let remote_pk = hex::decode("030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c1").unwrap();
        let funding_ws = new_2x2_multisig(&local_pk, &remote_pk);
        assert_eq!(hex::encode(funding_ws.data()), "5221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae");

        // Funding lockscript should be "0020c015c4a6be010e21657068fc2e6a9d02b27ebe4d490a25846f7237f104d1a3cd"
        let funding_lock_script = new_2x2_wsh_lock_script(&local_pk, &remote_pk);
        assert_eq!(hex::encode(funding_lock_script.data()), "0020c015c4a6be010e21657068fc2e6a9d02b27ebe4d490a25846f7237f104d1a3cd");

        let privkey_bytes = hex::decode("6bd078650fcee8444e4e09825227b801a1ca928debb750eb36e6d56124bb20e801").unwrap();
        assert_eq!(privkey_bytes.len(), 33);
        assert_eq!(privkey_bytes[32], 1);

        let sec = Secp256k1::new();
        let sk = SecretKey::from_slice(&sec, &privkey_bytes[0..32]).unwrap();
        // Pubkey (in compresses format) should have value: 03535b32d5eb0a6ed0982a0479bbadc9868d9836f6ba94dd5a63be16d875069184
        // This value was extracted from example transaction
        let pk = PublicKey::from_secret_key(&sec, &sk).unwrap();
        assert_eq!(hex::encode(&pk.serialize()[..]), "03535b32d5eb0a6ed0982a0479bbadc9868d9836f6ba94dd5a63be16d875069184");

        // Hash160 of compressed pubkey should be 3ca33c2e4446f4a305f23c80df8ad1afdcf652f9
        let pk_hash160 = Hash160::from_data(&pk.serialize()[..]).data();
        assert_eq!(hex::encode(&pk_hash160[..]), "3ca33c2e4446f4a305f23c80df8ad1afdcf652f9");

        // Change output script should be 00143ca33c2e4446f4a305f23c80df8ad1afdcf652f9
        let sc_change = v0_p2wpkh(&pk);
        assert_eq!(hex::encode(&sc_change.data()), "00143ca33c2e4446f4a305f23c80df8ad1afdcf652f9");

        let tx_out_funding = TxOut{
            value: 10_000_000,
            script_pubkey: funding_lock_script,
        };

        let tx_out_change = TxOut {
            value: 49_89_986_080,
            script_pubkey: sc_change,
        };

        let tx_in = TxIn {
            prev_hash: s2dh256("fd2105607605d2302994ffea703b09f66b6351816ee737a93e42a841ea20bbad"),
            prev_index: 0,
            script_sig: Script::new(),
            sequence: 4294967295,
            witness: vec![]
        };

        let mut tx = Transaction {
            version: 2,
            input: vec![tx_in],
            output: vec![tx_out_funding, tx_out_change],
            lock_time: 0
        };

        // Signature of transaction should be
        // 304502210090587b6201e166ad6af0227d3036a9454223d49a1f11839c1a362184340ef0240220577f7cd5cca78719405cbf1de7414ac027f0239ef6e214c90fcaab0454d84b3b[ALL]
        // We use deterministic signatures so it should be reproducible
        let sig_type = 1_u8; // SIGHASH_ALL
        let tx_sig_hash = tx.signature_hash(0, &p2pkh(&pk), sig_type as u32);
        let sig = sec.sign(&Message::from(tx_sig_hash.data()), &sk).unwrap();
        let mut sig_serialised = sig.serialize_der(&sec);
        assert_eq!(hex::encode(&sig_serialised), "304502210090587b6201e166ad6af0227d3036a9454223d49a1f11839c1a362184340ef0240220577f7cd5cca78719405cbf1de7414ac027f0239ef6e214c90fcaab0454d84b3b");
        // We need to add sigtype to the end of the signature
        sig_serialised.push(sig_type);
        tx.input[0].script_sig = p2pkh_unlock_script(&pk, &sig_serialised);

        // Hash of funding transaction should be 8984484a580b825b9972d7adb15050b3ab624ccd731946b3eeddb92f4e7ef6be
        assert_eq!(tx.txid().be_hex_string(), "8984484a580b825b9972d7adb15050b3ab624ccd731946b3eeddb92f4e7ef6be");
        // Transaction should be 0200000001adbb20ea41a8423ea937e76e8151636bf6093b70eaff942930d20576600521fd000000006b48304502210090587b6201e166ad6af0227d3036a9454223d49a1f11839c1a362184340ef0240220577f7cd5cca78719405cbf1de7414ac027f0239ef6e214c90fcaab0454d84b3b012103535b32d5eb0a6ed0982a0479bbadc9868d9836f6ba94dd5a63be16d875069184ffffffff028096980000000000220020c015c4a6be010e21657068fc2e6a9d02b27ebe4d490a25846f7237f104d1a3cd20256d29010000001600143ca33c2e4446f4a305f23c80df8ad1afdcf652f900000000
        let mut a = vec![];
        tx.consensus_encode(&mut RawEncoder::new(&mut a)).unwrap();
        assert_eq!(hex::encode(a), "0200000001adbb20ea41a8423ea937e76e8151636bf6093b70eaff942930d20576600521fd000000006b48304502210090587b6201e166ad6af0227d3036a9454223d49a1f11839c1a362184340ef0240220577f7cd5cca78719405cbf1de7414ac027f0239ef6e214c90fcaab0454d84b3b012103535b32d5eb0a6ed0982a0479bbadc9868d9836f6ba94dd5a63be16d875069184ffffffff028096980000000000220020c015c4a6be010e21657068fc2e6a9d02b27ebe4d490a25846f7237f104d1a3cd20256d29010000001600143ca33c2e4446f4a305f23c80df8ad1afdcf652f900000000");
    }

    #[test]
    fn test_get_obscuring_number() {
        let local_payment_basepoint = hex::decode("034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa").unwrap();
        let remote_payment_basepoint = hex::decode("032c0b7cf95324a07d05398b240174dc0c2be444d96b159aa6c7f7b1e668680991").unwrap();
        let obscuring_factor = get_obscuring_number(&local_payment_basepoint, &remote_payment_basepoint);
        assert_eq!(obscuring_factor, 0x2bb038521914);
    }

    #[test]
    fn test_get_locktime() {
        assert_eq!(get_locktime(42 ^ 0x2bb038521914), 542251326);
    }

    #[test]
    fn test_get_sequence() {
        assert_eq!(get_sequence(42 ^ 0x2bb038521914), 2150346808);
    }

    #[test]
    fn test_commit_tx_without_htlc() {
        let ex = get_example();

        // This transaction should be obtained
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8002c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311054a56a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400473044022051b75c73198c6deee1a875871c3961832909acd297c6b908d59e3319e5185a46022055c419379c5051a78d00dbbce11b5b664a0c22815fbcc6fcef6b1937c383693901483045022100f51d2e566a70ba740fc5d8c0f07b9b93d2ed741c3c0860c613173de7d39e7968022041376d520e9c0e1ad52248ddf4b22e12be8763007df977253ef45a4ca3bdb7c001475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

        // Transaction consists of:
        // 1. Version: 2
        // 2. Inputs:
        // 3. Outputs:
        // 4. Locktime:

        let obscuring_number = get_obscuring_number(&ex.local_payment_basepoint.serialize(), &ex.remote_payment_basepoint.serialize());
        assert_eq!(obscuring_number, ex.obscuring_factor);

        let remote_funding_pk = PublicKey::from_secret_key(&Secp256k1::new(), &ex.internal.remote_funding_privkey).unwrap();
        assert_eq!(remote_funding_pk, ex.remote_funding_pubkey);

        let obscured_commit_number = ex.commitment_number ^ obscuring_number;
        assert_eq!(get_sequence(obscured_commit_number), 2150346808);
        assert_eq!(get_locktime(obscured_commit_number), 542251326);

        let sequence = get_sequence(obscured_commit_number);
        let locktime = get_locktime(obscured_commit_number);

        let to_local_script_ex = s2script("63210212a140cd0c6539d07cd08dfe09984dec3251ea808b892efeac3ede9402bf2b1967029000b2752103fd5960528dc152014952efdb702a88f71e3c1653b2314431701ec77e57fde83c68ac");
        let to_local = to_local_script(&ex.local_delayedpubkey, ex.local_delay as u64, &ex.local_revocation_pubkey);
        assert_eq!(to_local, to_local_script_ex);
        let to_local_lock_script = to_local.to_v0_p2wsh();
        assert_eq!(to_local_lock_script, s2script("00204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e"));

        let to_remote_lock_script = v0_p2wpkh(&ex.remotepubkey);
        assert_eq!(to_remote_lock_script, s2script("0014ccf1af2f2aabee14bb40fa3851ab2301de843110"));

        let funding_lock_script = new_2x2_multisig(&ex.local_funding_pubkey.serialize(), &ex.remote_funding_pubkey.serialize());
        assert_eq!(funding_lock_script, s2script("5221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae"));

        let mut tx = Transaction{
            version: 2,
            input: vec![TxIn{
                prev_hash: ex.funding_tx_id,
                prev_index: ex.funding_output_index as u32,
                sequence: sequence as u32,
                script_sig: Script::new(),
                witness: vec![]
            }],
            output: vec![
                TxOut{
                    value: 3000000,
                    script_pubkey: to_remote_lock_script
                },
                TxOut{
                    value: 6989140,
                    script_pubkey: to_local_lock_script
                }
            ],
            lock_time: locktime as u32
        };

        // Check that all fields except witness is correct
        assert_tx_eq(&tx, &example_tx, true);

        let sec = Secp256k1::new();
        let tx_sig_hash = bip143::SighashComponents::new(&tx).sighash_all(&tx.input[0], &funding_lock_script, ex.funding_amount_satoshi as u64);
        let sig_local = sec.sign(&Message::from(tx_sig_hash.data()), &ex.local_funding_privkey).unwrap();
        let mut sig_local_serialised = sig_local.serialize_der(&sec);
        assert_eq!(hex::encode(&sig_local_serialised), "3044022051b75c73198c6deee1a875871c3961832909acd297c6b908d59e3319e5185a46022055c419379c5051a78d00dbbce11b5b664a0c22815fbcc6fcef6b1937c3836939");
        sig_local_serialised.push(1);

        let sig_remote = sec.sign(&Message::from(tx_sig_hash.data()), &ex.internal.remote_funding_privkey).unwrap();
        let mut sig_remote_serialised = sig_remote.serialize_der(&sec);
        assert_eq!(hex::encode(&sig_remote_serialised), "3045022100f51d2e566a70ba740fc5d8c0f07b9b93d2ed741c3c0860c613173de7d39e7968022041376d520e9c0e1ad52248ddf4b22e12be8763007df977253ef45a4ca3bdb7c0");
        sig_remote_serialised.push(1);

        tx.input[0].witness = vec![
            vec![],
            sig_local_serialised,
            sig_remote_serialised,
            funding_lock_script.data()
        ];
        assert_tx_eq(&tx, &example_tx, false);

        let mut a = vec![];
        tx.consensus_encode(&mut RawEncoder::new(&mut a)).unwrap();
        assert_eq!(hex::encode(a), "02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8002c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311054a56a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400473044022051b75c73198c6deee1a875871c3961832909acd297c6b908d59e3319e5185a46022055c419379c5051a78d00dbbce11b5b664a0c22815fbcc6fcef6b1937c383693901483045022100f51d2e566a70ba740fc5d8c0f07b9b93d2ed741c3c0860c613173de7d39e7968022041376d520e9c0e1ad52248ddf4b22e12be8763007df977253ef45a4ca3bdb7c001475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
    }

    #[test]
    fn test_commit_tx_with_five_htlc() {
        let ex = get_example();

        // name: commitment tx with all five HTLCs untrimmed (minimum feerate)
        let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8007e80300000000000022002052bfef0479d7b293c27e0f1eb294bea154c63a3294ef092c19af51409bce0e2ad007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110e0a06a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e04004730440220275b0c325a5e9355650dc30c0eccfbc7efb23987c24b556b9dfdd40effca18d202206caceb2c067836c51f296740c7ae807ffcbfbf1dd3a0d56b6de9a5b247985f060147304402204fd4928835db1ccdfc40f5c78ce9bd65249b16348df81f0c44328dcdefc97d630220194d3869c38bc732dd87d13d2958015e2fc16829e74cd4377f84d215c0b7060601475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

        //    Transaction output order (from decoded example tx):
        //    (HTLC 0)
        //    (HTLC 2)
        //    (HTLC 1)
        //    (HTLC 3)
        //    (HTLC 4)
        //    To remote
        //    To local

        let obscuring_number = get_obscuring_number(&ex.local_payment_basepoint.serialize(), &ex.remote_payment_basepoint.serialize());
        assert_eq!(obscuring_number, ex.obscuring_factor);
        let obscured_commit_number = ex.commitment_number ^ obscuring_number;
        assert_eq!(get_sequence(obscured_commit_number), 2150346808);
        assert_eq!(get_locktime(obscured_commit_number), 542251326);

        let sequence = get_sequence(obscured_commit_number);
        let locktime = get_locktime(obscured_commit_number);

        // Try to recreate output 0
        assert_eq!(hex::encode(Hash160::from_data(&(ex.local_revocation_pubkey.serialize())).data()), "14011f7254d96b819c76986c277d115efce6f7b5");
        assert_eq!(hex::encode(Hash160::from_data(&ex.htlcs[0].payment_preimage).data()), "b8bcb07f6344b42ab04250c86a6e8b75d3fdbbc6");

        // It seems like it is remotepubkey
        let remote_htlc_pubkey = s2pubkey("0394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b");
        // It seems like it is localpubkey
        let local_htlc_pubkey = s2pubkey("030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e7");

        let htlc0_script = accepted_htlc(&ex.local_revocation_pubkey, &remote_htlc_pubkey, &local_htlc_pubkey, sha256(&ex.htlcs[0].payment_preimage), ex.htlcs[0].expiry as u32);
        assert_eq!(htlc0_script, s2script("76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c8201208763a914b8bcb07f6344b42ab04250c86a6e8b75d3fdbbc688527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae677502f401b175ac6868"));

        let htlc2_script = offered_htlc(&ex.local_revocation_pubkey, &remote_htlc_pubkey, &local_htlc_pubkey, sha256(&ex.htlcs[2].payment_preimage));
        assert_eq!(htlc2_script, s2script("76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c820120876475527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae67a914b43e1b38138a41b37f7cd9a1d274bc63e3a9b5d188ac6868"));

        let htlc1_script = accepted_htlc(&ex.local_revocation_pubkey, &remote_htlc_pubkey, &local_htlc_pubkey, sha256(&ex.htlcs[1].payment_preimage), ex.htlcs[1].expiry as u32);
        assert_eq!(htlc1_script, s2script("76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c8201208763a9144b6b2e5444c2639cc0fb7bcea5afba3f3cdce23988527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae677502f501b175ac6868"));

        let htlc3_script = offered_htlc(&ex.local_revocation_pubkey, &remote_htlc_pubkey, &local_htlc_pubkey, sha256(&ex.htlcs[3].payment_preimage));
        assert_eq!(htlc3_script, s2script("76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c820120876475527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae67a9148a486ff2e31d6158bf39e2608864d63fefd09d5b88ac6868"));

        let htlc4_script = accepted_htlc(&ex.local_revocation_pubkey, &remote_htlc_pubkey, &local_htlc_pubkey, sha256(&ex.htlcs[4].payment_preimage), ex.htlcs[4].expiry as u32);
        assert_eq!(htlc4_script, s2script("76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c8201208763a91418bc1a114ccf9c052d3d23e28d3b0a9d1227434288527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae677502f801b175ac6868"));

        let to_local = to_local_script(&ex.local_delayedpubkey, ex.local_delay as u64, &ex.local_revocation_pubkey);
        let to_local_lock_script = to_local.to_v0_p2wsh();
        assert_eq!(to_local_lock_script, s2script("00204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e"));

        let to_remote_lock_script = v0_p2wpkh(&ex.remotepubkey);
        assert_eq!(to_remote_lock_script, s2script("0014ccf1af2f2aabee14bb40fa3851ab2301de843110"));

        let funding_lock_script = new_2x2_multisig(&ex.local_funding_pubkey.serialize(), &ex.remote_funding_pubkey.serialize());
        assert_eq!(funding_lock_script, s2script("5221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae"));

        let mut tx = Transaction{
            version: 2,
            input: vec![TxIn{
                prev_hash: ex.funding_tx_id,
                prev_index: ex.funding_output_index as u32,
                sequence: sequence as u32,
                script_sig: Script::new(),
                witness: vec![]
            }],
            output: vec![
                TxOut{
                    value: (ex.htlcs[0].amount_msat / 1000) as u64,
                    script_pubkey: htlc0_script.to_v0_p2wsh()
                },
                TxOut{
                    value: (ex.htlcs[2].amount_msat / 1000) as u64,
                    script_pubkey: htlc2_script.to_v0_p2wsh()
                },
                TxOut{
                    value: (ex.htlcs[1].amount_msat / 1000) as u64,
                    script_pubkey: htlc1_script.to_v0_p2wsh()
                },
                TxOut{
                    value: (ex.htlcs[3].amount_msat / 1000) as u64,
                    script_pubkey: htlc3_script.to_v0_p2wsh()
                },
                TxOut{
                    value: (ex.htlcs[4].amount_msat / 1000) as u64,
                    script_pubkey: htlc4_script.to_v0_p2wsh()
                },
                TxOut{
                    value: 3000000,
                    script_pubkey: to_remote_lock_script
                },
                TxOut{
                    value: 6988000,
                    script_pubkey: to_local_lock_script
                }
            ],
            lock_time: locktime as u32
        };

        // Check that all fields except witness is correct
        assert_tx_eq(&tx, &example_tx, true);

        let sec = Secp256k1::new();
        let tx_sig_hash = bip143::SighashComponents::new(&tx).sighash_all(&tx.input[0], &funding_lock_script, ex.funding_amount_satoshi as u64);
        let sig_local = sec.sign(&Message::from(tx_sig_hash.data()), &ex.local_funding_privkey).unwrap();
        let mut sig_local_serialised = sig_local.serialize_der(&sec);
        assert_eq!(hex::encode(&sig_local_serialised), "30440220275b0c325a5e9355650dc30c0eccfbc7efb23987c24b556b9dfdd40effca18d202206caceb2c067836c51f296740c7ae807ffcbfbf1dd3a0d56b6de9a5b247985f06");
        sig_local_serialised.push(1);

        let sig_remote = sec.sign(&Message::from(tx_sig_hash.data()), &ex.internal.remote_funding_privkey).unwrap();
        let mut sig_remote_serialised = sig_remote.serialize_der(&sec);
        assert_eq!(hex::encode(&sig_remote_serialised), "304402204fd4928835db1ccdfc40f5c78ce9bd65249b16348df81f0c44328dcdefc97d630220194d3869c38bc732dd87d13d2958015e2fc16829e74cd4377f84d215c0b70606");
        sig_remote_serialised.push(1);

        tx.input[0].witness = vec![
            vec![],
            sig_local_serialised,
            sig_remote_serialised,
            funding_lock_script.data()
        ];
        assert_tx_eq(&tx, &example_tx, false);

        let mut a = vec![];
        tx.consensus_encode(&mut RawEncoder::new(&mut a)).unwrap();
        assert_eq!(hex::encode(a), "02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8007e80300000000000022002052bfef0479d7b293c27e0f1eb294bea154c63a3294ef092c19af51409bce0e2ad007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110e0a06a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e04004730440220275b0c325a5e9355650dc30c0eccfbc7efb23987c24b556b9dfdd40effca18d202206caceb2c067836c51f296740c7ae807ffcbfbf1dd3a0d56b6de9a5b247985f060147304402204fd4928835db1ccdfc40f5c78ce9bd65249b16348df81f0c44328dcdefc97d630220194d3869c38bc732dd87d13d2958015e2fc16829e74cd4377f84d215c0b7060601475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");
    }

}

