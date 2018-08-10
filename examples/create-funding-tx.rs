extern crate bitcoin;
extern crate hex;
extern crate secp256k1;
extern crate crypto;
extern crate lpd;

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

use lpd::tools::{s2dh256, s2byte32, s2pubkey, s2privkey, new_2x2_multisig, new_2x2_wsh_lock_script, v0_p2wpkh, p2pkh, p2pkh_unlock_script, s2script};
use lpd::bip69;

const HTLC_TIMEOUT_WEIGHT: i64 = 663;
const HTLC_SUCCESS_WEIGHT: i64 = 703;
const BASE_COMMITMENT_WEIGHT: i64 = 724;
const PER_HTLC_COMMITMENT_WEIGHT: i64 = 172;

//+funding_tx_id: 8984484a580b825b9972d7adb15050b3ab624ccd731946b3eeddb92f4e7ef6be
//+funding_output_index: 0
//+funding_amount_satoshi: 10000000
//+commitment_number: 42
//+local_delay: 144
//+local_dust_limit_satoshi: 546
//htlc 0 direction: remote->local
//htlc 0 amount_msat: 1000000
//htlc 0 expiry: 500
//htlc 0 payment_preimage: 0000000000000000000000000000000000000000000000000000000000000000

//htlc 1 direction: remote->local
//htlc 1 amount_msat: 2000000
//htlc 1 expiry: 501
//htlc 1 payment_preimage: 0101010101010101010101010101010101010101010101010101010101010101

//htlc 2 direction: local->remote
//htlc 2 amount_msat: 2000000
//htlc 2 expiry: 502
//htlc 2 payment_preimage: 0202020202020202020202020202020202020202020202020202020202020202

//htlc 3 direction: local->remote
//htlc 3 amount_msat: 3000000
//htlc 3 expiry: 503
//htlc 3 payment_preimage: 0303030303030303030303030303030303030303030303030303030303030303

//htlc 4 direction: remote->local
//htlc 4 amount_msat: 4000000
//htlc 4 expiry: 504
//htlc 4 payment_preimage: 0404040404040404040404040404040404040404040404040404040404040404


//local_payment_basepoint: 034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa
//remote_payment_basepoint: 032c0b7cf95324a07d05398b240174dc0c2be444d96b159aa6c7f7b1e668680991
//# obscured commitment transaction number = 0x2bb038521914 ^ 42

//local_funding_privkey: 30ff4956bbdd3222d44cc5e8a1261dab1e07957bdac5ae88fe3261ef321f374901
//local_funding_pubkey: 023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb
//remote_funding_pubkey: 030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c1
//local_privkey: bb13b121cdc357cd2e608b0aea294afca36e2b34cf958e2e6451a2f27469449101
//localpubkey: 030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e7
//remotepubkey: 0394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b
//local_delayedpubkey: 03fd5960528dc152014952efdb702a88f71e3c1653b2314431701ec77e57fde83c
//local_revocation_pubkey: 0212a140cd0c6539d07cd08dfe09984dec3251ea808b892efeac3ede9402bf2b19
//# funding wscript = 5221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae

//INTERNAL: remote_funding_privkey: 1552dfba4f6cf29a62a0af13c8d6981d36d0ef8d61ba10fb0fe90da7634d7e130101
//INTERNAL: local_payment_basepoint_secret: 111111111111111111111111111111111111111111111111111111111111111101
//INTERNAL: remote_revocation_basepoint_secret: 222222222222222222222222222222222222222222222222222222222222222201
//INTERNAL: local_delayed_payment_basepoint_secret: 333333333333333333333333333333333333333333333333333333333333333301
//INTERNAL: remote_payment_basepoint_secret: 444444444444444444444444444444444444444444444444444444444444444401
//x_local_per_commitment_secret: 1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a0908070605040302010001
//# From remote_revocation_basepoint_secret
//INTERNAL: remote_revocation_basepoint: 02466d7fcae563e5cb09a0d1870bb580344804617879a14949cf22285f1bae3f27
//# From local_delayed_payment_basepoint_secret
//INTERNAL: local_delayed_payment_basepoint: 023c72addb4fdf09af94f0c94d7fe92a386a7e70cf8a1d85916386bb2535c7b1b1
//INTERNAL: local_per_commitment_point: 025f7117a78150fe2ef97db7cfc83bd57b2e2c0d0dd25eaf467a4a1c2a45ce1486
//INTERNAL: remote_privkey: 8deba327a7cc6d638ab0eb025770400a6184afcba6713c210d8d10e199ff2fda01
//# From local_delayed_payment_basepoint_secret, local_per_commitment_point and local_delayed_payment_basepoint
//INTERNAL: local_delayed_privkey: adf3464ce9c2f230fd2582fda4c6965e4993ca5524e8c9580e3df0cf226981ad01

// Contains data internal to spec. Private keys for remote node, ...
struct Internal {
    remote_funding_privkey: SecretKey,
    local_payment_basepoint_secret: SecretKey,
    remote_revocation_basepoint_secret: SecretKey,
    local_delayed_payment_basepoint_secret: SecretKey,
    remote_payment_basepoint_secret: SecretKey,
    x_local_per_commitment_secret: SecretKey,
    remote_revocation_basepoint: PublicKey,
    local_delayed_payment_basepoint: PublicKey,
    local_per_commitment_point: PublicKey,
    remote_privkey: SecretKey,
    local_delayed_privkey: SecretKey,
}

struct HtlcExample {
    direction: String,
    amount_msat: i64,
    expiry: i32,
    payment_preimage: [u8; 32]
}

impl HtlcExample {
    fn to_htlc(&self) -> HTLC {
        let direction = match self.direction.as_ref() {
            "remote->local" => HTLCDirection::Accepted,
            "local->remote" => HTLCDirection::Offered,
            _ => panic!("unknown htlc direction in example data"),
        };
        let htlc = HTLC{
            direction: direction,
            amount_msat: self.amount_msat,
            payment_hash: sha256(&self.payment_preimage),
            expiry: self.expiry,
        };
        return htlc;
    }
}

struct SpecExample {
    funding_tx_id: Sha256dHash,
    funding_output_index: i32,
    funding_amount_satoshi: i64,
    commitment_number: u64,
    local_delay: i32,
    local_dust_limit_satoshi: i64,
    htlcs: Vec<HtlcExample>,
    local_payment_basepoint: PublicKey,
    remote_payment_basepoint: PublicKey,
    obscuring_factor: u64,
    local_funding_privkey: SecretKey,
    local_funding_pubkey: PublicKey,
    remote_funding_pubkey: PublicKey,
    local_privkey: SecretKey,
    localpubkey: PublicKey,
    remotepubkey: PublicKey,
    local_delayedpubkey: PublicKey,
    local_revocation_pubkey: PublicKey,
    internal: Internal
}

fn get_example() -> SpecExample {
    let ex = SpecExample {
        funding_tx_id: s2dh256("8984484a580b825b9972d7adb15050b3ab624ccd731946b3eeddb92f4e7ef6be"),
        funding_output_index: 0,
        funding_amount_satoshi: 10000000,
        commitment_number: 42,
        local_delay: 144,
        local_dust_limit_satoshi: 546,
        htlcs: vec![
            HtlcExample {
                direction: "remote->local".to_owned(),
                amount_msat: 1000000,
                expiry: 500,
                payment_preimage: s2byte32("0000000000000000000000000000000000000000000000000000000000000000")
            },
            HtlcExample {
                direction: "remote->local".to_owned(),
                amount_msat: 2000000,
                expiry: 501,
                payment_preimage: s2byte32("0101010101010101010101010101010101010101010101010101010101010101")
            },
            HtlcExample {
                direction: "local->remote".to_owned(),
                amount_msat: 2000000,
                expiry: 502,
                payment_preimage: s2byte32("0202020202020202020202020202020202020202020202020202020202020202")
            },
            HtlcExample {
                direction: "local->remote".to_owned(),
                amount_msat: 3000000,
                expiry: 503,
                payment_preimage: s2byte32("0303030303030303030303030303030303030303030303030303030303030303")
            },
            HtlcExample {
                direction: "remote->local".to_owned(),
                amount_msat: 4000000,
                expiry: 504,
                payment_preimage: s2byte32("0404040404040404040404040404040404040404040404040404040404040404")
            }
        ],
        local_payment_basepoint: s2pubkey("034f355bdcb7cc0af728ef3cceb9615d90684bb5b2ca5f859ab0f0b704075871aa"),
        remote_payment_basepoint: s2pubkey("032c0b7cf95324a07d05398b240174dc0c2be444d96b159aa6c7f7b1e668680991"),
        obscuring_factor: 0x2bb038521914,
        local_funding_privkey: s2privkey("30ff4956bbdd3222d44cc5e8a1261dab1e07957bdac5ae88fe3261ef321f374901"),
        local_funding_pubkey: s2pubkey("023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb"),
        remote_funding_pubkey: s2pubkey("030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c1"),
        local_privkey: s2privkey("bb13b121cdc357cd2e608b0aea294afca36e2b34cf958e2e6451a2f27469449101"),
        localpubkey: s2pubkey("030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e7"),
        remotepubkey: s2pubkey("0394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b"),
        local_delayedpubkey: s2pubkey("03fd5960528dc152014952efdb702a88f71e3c1653b2314431701ec77e57fde83c"),
        local_revocation_pubkey: s2pubkey("0212a140cd0c6539d07cd08dfe09984dec3251ea808b892efeac3ede9402bf2b19"),
        internal: Internal{
            remote_funding_privkey: s2privkey("1552dfba4f6cf29a62a0af13c8d6981d36d0ef8d61ba10fb0fe90da7634d7e1301"),
            local_payment_basepoint_secret: s2privkey("111111111111111111111111111111111111111111111111111111111111111101"),
            remote_revocation_basepoint_secret: s2privkey("222222222222222222222222222222222222222222222222222222222222222201"),
            local_delayed_payment_basepoint_secret: s2privkey("333333333333333333333333333333333333333333333333333333333333333301"),
            remote_payment_basepoint_secret: s2privkey("444444444444444444444444444444444444444444444444444444444444444401"),
            x_local_per_commitment_secret: s2privkey("1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a0908070605040302010001"),
            remote_revocation_basepoint: s2pubkey("02466d7fcae563e5cb09a0d1870bb580344804617879a14949cf22285f1bae3f27"),
            local_delayed_payment_basepoint: s2pubkey("023c72addb4fdf09af94f0c94d7fe92a386a7e70cf8a1d85916386bb2535c7b1b1"),
            local_per_commitment_point: s2pubkey("025f7117a78150fe2ef97db7cfc83bd57b2e2c0d0dd25eaf467a4a1c2a45ce1486"),
            remote_privkey: s2privkey("8deba327a7cc6d638ab0eb025770400a6184afcba6713c210d8d10e199ff2fda01"),
            local_delayed_privkey: s2privkey("adf3464ce9c2f230fd2582fda4c6965e4993ca5524e8c9580e3df0cf226981ad01"),
        }
    };
    return ex;
}

fn example_from_spec() {
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

fn s2tx(s: &str) -> Transaction {
    let tx_bytes = hex::decode(s).unwrap();
    let tx = Transaction::consensus_decode(&mut RawDecoder::new(&tx_bytes[..])).unwrap();
    return tx;
}

// Get sequence number from obscured commit number
// upper 8 bits are 0x80,
// lower 24 bits are upper 24 bits of the obscured commitment transaction number
fn get_sequence(x: u64) -> u64 {
    return (0x80 << 24) + (x >> 24);
}

// Get sequence number from obscured sequence number
// upper 8 bits are 0x20,
// lower 24 bits are the lower 24 bits of the obscured commitment transaction number
fn get_locktime(x: u64) -> u64 {
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
fn to_local_script(local_delayedpubkey: &PublicKey, to_self_delay: u64, revocationpubkey: &PublicKey) -> Script {
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

fn spec_ex_1() {
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

fn assert_tx_eq(tx1: &Transaction, tx2: &Transaction, ignore_witness: bool) {
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

fn get_obscuring_number(local_payment_basepoint: &[u8], remote_payment_basepoint: &[u8]) -> u64 {

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

fn sha256(x: &[u8]) -> [u8; 32] {
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
fn offered_htlc(revocationpubkey: &PublicKey, remote_htlcpubkey: &PublicKey, local_htlcpubkey: &PublicKey, payment_hash: [u8; 32]) -> Script {
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
fn accepted_htlc(revocationpubkey: &PublicKey, remote_htlcpubkey: &PublicKey, local_htlcpubkey: &PublicKey, payment_hash: [u8; 32], cltv_expiry: u32) -> Script {
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

fn spec_ex_2() {
    let ex = get_example();

    // name: commitment tx with all five HTLCs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8007e80300000000000022002052bfef0479d7b293c27e0f1eb294bea154c63a3294ef092c19af51409bce0e2ad007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110e0a06a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e04004730440220275b0c325a5e9355650dc30c0eccfbc7efb23987c24b556b9dfdd40effca18d202206caceb2c067836c51f296740c7ae807ffcbfbf1dd3a0d56b6de9a5b247985f060147304402204fd4928835db1ccdfc40f5c78ce9bd65249b16348df81f0c44328dcdefc97d630220194d3869c38bc732dd87d13d2958015e2fc16829e74cd4377f84d215c0b7060601475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

//    Transaction outputs:
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

    println!("{:?}", bip69::TransactionReordering::from_tx_to_bip69(&tx));
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

enum HTLCDirection {
    Accepted,
    Offered,
}

struct HTLC {
    direction: HTLCDirection,
    amount_msat: i64,
    expiry: i32,
    payment_hash: [u8; 32]
}

struct CommitTx {
    funding_amount: i64,

    local_feerate_per_kw: i64,
    dust_limit_satoshi: i64,

    to_local_msat: i64,
    to_remote_msat: i64,

    obscured_commit_number: u64,

    local_htlc_pubkey: PublicKey,
    remote_htlc_pubkey: PublicKey,

    local_revocation_pubkey: PublicKey,
    local_delayedpubkey: PublicKey,
    local_delay: u64,

    remotepubkey: PublicKey,

    funding_tx_id: Sha256dHash,
    funding_output_index: u32,

    htlcs: Vec<HTLC>,
}

impl CommitTx {
    fn get_tx(&self) -> Transaction {
        let sequence = get_sequence(self.obscured_commit_number);
        let locktime = get_locktime(self.obscured_commit_number);

        let mut tx = Transaction{
            version: 2,
            input: vec![TxIn{
                prev_hash: self.funding_tx_id,
                prev_index: self.funding_output_index,
                sequence: sequence as u32,
                script_sig: Script::new(),
                witness: vec![]
            }],
            output: vec![
            ],
            lock_time: locktime as u32
        };

        let mut weight: i64 = BASE_COMMITMENT_WEIGHT;
        for h in &self.htlcs {
            if self.is_htlc_trimmed(h) {
                continue
            }
            weight += PER_HTLC_COMMITMENT_WEIGHT;
            let lock_script = match h.direction {
                HTLCDirection::Accepted => accepted_htlc(&self.local_revocation_pubkey, &self.remote_htlc_pubkey, &self.local_htlc_pubkey, h.payment_hash, h.expiry as u32),
                HTLCDirection::Offered => offered_htlc(&self.local_revocation_pubkey, &self.remote_htlc_pubkey, &self.local_htlc_pubkey, h.payment_hash),
            };
            tx.output.push(TxOut{
                value: (h.amount_msat / 1000) as u64,
                script_pubkey: lock_script.to_v0_p2wsh(),
            })
        }

        let base_fee = (weight * self.local_feerate_per_kw) / 1000;
        // TODO(mkl): who pays it
        // TODO(mkl): what happens if it is negative
        // Assume that local pays fee
        let mut to_local = (self.to_local_msat / 1000) - base_fee;
        let mut to_remote = self.to_remote_msat / 1000;

        if to_local < 0 {
            to_local = 0;
            println!("too high fee encountered, to_local < 0, set to 0");
        }

        if to_remote < 0 {
            to_remote = 0;
            println!("too high fee encountered, to_remote < 0, set to 0");
        }

        // To self output
        if to_local >= self.dust_limit_satoshi {
            tx.output.push(TxOut{
                value: to_local as u64,
                script_pubkey: to_local_script(&self.local_delayedpubkey, self.local_delay as u64, &self.local_revocation_pubkey).to_v0_p2wsh(),
            });
        }

        // To remote output
        if to_remote >= self.dust_limit_satoshi {
            tx.output.push(TxOut{
                value: to_remote as u64,
                script_pubkey: v0_p2wpkh(&self.remotepubkey),
            });
        }

        bip69::reorder_tx(&mut tx);

        return tx;
    }

    fn htlc_timeout_fee(&self) -> i64 {
        return self.local_feerate_per_kw * HTLC_TIMEOUT_WEIGHT / 1000;
    }

    fn htlc_success_fee(&self) -> i64 {
        return self.local_feerate_per_kw * HTLC_SUCCESS_WEIGHT / 1000;
    }

    fn is_htlc_trimmed(&self, h: &HTLC) -> bool {
        let required = self.dust_limit_satoshi + match h.direction {
            HTLCDirection::Accepted => self.htlc_success_fee(),
            HTLCDirection::Offered => self.htlc_timeout_fee(),
        };
        return (h.amount_msat / 1000) < required;
    }
}

fn spec_ex_1_1() {
    let ex = get_example();

    // name: simple commitment tx with no HTLCs
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8002c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311054a56a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400473044022051b75c73198c6deee1a875871c3961832909acd297c6b908d59e3319e5185a46022055c419379c5051a78d00dbbce11b5b664a0c22815fbcc6fcef6b1937c383693901483045022100f51d2e566a70ba740fc5d8c0f07b9b93d2ed741c3c0860c613173de7d39e7968022041376d520e9c0e1ad52248ddf4b22e12be8763007df977253ef45a4ca3bdb7c001475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 15000,
        dust_limit_satoshi: 546,

        to_local_msat: 7000000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_2_1() {
    let ex = get_example();

    // name: commitment tx with all five HTLCs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8007e80300000000000022002052bfef0479d7b293c27e0f1eb294bea154c63a3294ef092c19af51409bce0e2ad007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110e0a06a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e04004730440220275b0c325a5e9355650dc30c0eccfbc7efb23987c24b556b9dfdd40effca18d202206caceb2c067836c51f296740c7ae807ffcbfbf1dd3a0d56b6de9a5b247985f060147304402204fd4928835db1ccdfc40f5c78ce9bd65249b16348df81f0c44328dcdefc97d630220194d3869c38bc732dd87d13d2958015e2fc16829e74cd4377f84d215c0b7060601475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 0,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_3_1() {
    let ex = get_example();

    // name: commitment tx with seven outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8007e80300000000000022002052bfef0479d7b293c27e0f1eb294bea154c63a3294ef092c19af51409bce0e2ad007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110e09c6a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040048304502210094bfd8f5572ac0157ec76a9551b6c5216a4538c07cd13a51af4a54cb26fa14320220768efce8ce6f4a5efac875142ff19237c011343670adf9c7ac69704a120d116301483045022100a5c01383d3ec646d97e40f44318d49def817fcd61a0ef18008a665b3e151785502203e648efddd5838981ef55ec954be69c4a652d021e6081a100d034de366815e9b01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 647,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_4_1() {
    let ex = get_example();

    // name: commitment tx with six outputs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8006d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431104e9d6a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400483045022100a2270d5950c89ae0841233f6efea9c951898b301b2e89e0adbd2c687b9f32efa02207943d90f95b9610458e7c65a576e149750ff3accaacad004cd85e70b235e27de01473044022072714e2fbb93cdd1c42eb0828b4f2eff143f717d8f26e79d6ada4f0dcb681bbe02200911be4e5161dd6ebe59ff1c58e1997c4aea804f81db6b698821db6093d7b05701475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 648,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_5_1() {
    let ex = get_example();

    // name: commitment tx with six outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8006d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5d007000000000000220020748eba944fedc8827f6b06bc44678f93c0f9e6078b35c6331ed31e75f8ce0c2db80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311077956a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402203ca8f31c6a47519f83255dc69f1894d9a6d7476a19f498d31eaf0cd3a85eeb63022026fd92dc752b33905c4c838c528b692a8ad4ced959990b5d5ee2ff940fa90eea01473044022001d55e488b8b035b2dd29d50b65b530923a416d47f377284145bc8767b1b6a75022019bb53ddfe1cefaf156f924777eaaf8fdca1810695a7d0a247ad2afba8232eb401475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 2069,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_6_1() {
    let ex = get_example();

    //name: commitment tx with five outputs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8005d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110da966a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e04004730440220443cb07f650aebbba14b8bc8d81e096712590f524c5991ac0ed3bbc8fd3bd0c7022028a635f548e3ca64b19b69b1ea00f05b22752f91daf0b6dab78e62ba52eb7fd001483045022100f2377f7a67b7fc7f4e2c0c9e3a7de935c32417f5668eda31ea1db401b7dc53030220415fdbc8e91d0f735e70c21952342742e25249b0d062d43efbfc564499f3752601475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 2070,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_7_1() {
    let ex = get_example();

    // name: commitment tx with five outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8005d007000000000000220020403d394747cae42e98ff01734ad5c08f82ba123d3d9a620abda88989651e2ab5b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311040966a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402203b1b010c109c2ecbe7feb2d259b9c4126bd5dc99ee693c422ec0a5781fe161ba0220571fe4e2c649dea9c7aaf7e49b382962f6a3494963c97d80fef9a430ca3f706101483045022100d33c4e541aa1d255d41ea9a3b443b3b822ad8f7f86862638aac1f69f8f760577022007e2a18e6931ce3d3a804b1c78eda1de17dbe1fb7a95488c9a4ec8620395334801475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 2194,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_8_1() {
    let ex = get_example();

    // name: commitment tx with four outputs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8004b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110b8976a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402203b12d44254244b8ff3bb4129b0920fd45120ab42f553d9976394b099d500c99e02205e95bb7a3164852ef0c48f9e0eaf145218f8e2c41251b231f03cbdc4f29a54290147304402205e2f76d4657fb732c0dfc820a18a7301e368f5799e06b7828007633741bda6df0220458009ae59d0c6246065c419359e05eb2a4b4ef4a1b310cc912db44eb792429801475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 2195,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_9_1() {
    let ex = get_example();

    //name: commitment tx with four outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8004b80b000000000000220020c20b5d1f8584fd90443e7b7b720136174fa4b9333c261d04dbbd012635c0f419a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431106f916a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402200e930a43c7951162dc15a2b7344f48091c74c70f7024e7116e900d8bcfba861c022066fa6cbda3929e21daa2e7e16a4b948db7e8919ef978402360d1095ffdaff7b001483045022100c1a3b0b60ca092ed5080121f26a74a20cec6bdee3f8e47bae973fcdceb3eda5502207d467a9873c939bf3aa758014ae67295fedbca52412633f7e5b2670fc7c381c101475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 3702,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_10_1() {
    let ex = get_example();

    //name: commitment tx with three outputs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8003a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110eb936a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400473044022047305531dd44391dce03ae20f8735005c615eb077a974edb0059ea1a311857d602202e0ed6972fbdd1e8cb542b06e0929bc41b2ddf236e04cb75edd56151f4197506014830450221008b7c191dd46893b67b628e618d2dc8e81169d38bade310181ab77d7c94c6675e02203b4dd131fd7c9deb299560983dcdc485545c98f989f7ae8180c28289f9e6bdb001475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 3703,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_11_1() {
    let ex = get_example();

    //name: commitment tx with three outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8003a00f0000000000002200208c48d15160397c9731df9bc3b236656efb6665fbfe92b4a6878e88a499f741c4c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110ae8f6a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e040047304402206a2679efa3c7aaffd2a447fd0df7aba8792858b589750f6a1203f9259173198a022008d52a0e77a99ab533c36206cb15ad7aeb2aa72b93d4b571e728cb5ec2f6fe260147304402206d6cb93969d39177a09d5d45b583f34966195b77c7e585cf47ac5cce0c90cefb022031d71ae4e33a4e80df7f981d696fbdee517337806a3c7138b7491e2cbb077a0e01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 4914,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_12_1() {
    let ex = get_example();

    // name: commitment tx with two outputs untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8002c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de843110fa926a00000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80e0400483045022100a012691ba6cea2f73fa8bac37750477e66363c6d28813b0bb6da77c8eb3fb0270220365e99c51304b0b1a6ab9ea1c8500db186693e39ec1ad5743ee231b0138384b90147304402200769ba89c7330dfa4feba447b6e322305f12ac7dac70ec6ba997ed7c1b598d0802204fe8d337e7fee781f9b7b1a06e580b22f4f79d740059560191d7db53f876555201475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 4915,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_13_1() {
    let ex = get_example();

    // name: commitment tx with two outputs untrimmed (maximum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b800222020000000000002200204adb4e2f00643db396dd120d4e7dc17625f5f2c11a40d857accc862d6b7dd80ec0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de84311004004730440220514f977bf7edc442de8ce43ace9686e5ebdc0f893033f13e40fb46c8b8c6e1f90220188006227d175f5c35da0b092c57bea82537aed89f7778204dc5bacf4f29f2b901473044022037f83ff00c8e5fb18ae1f918ffc24e54581775a20ff1ae719297ef066c71caa9022039c529cccd89ff6c5ed1db799614533844bd6d101da503761c45c713996e3bbd01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 9651180,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_14_1() {
    let ex = get_example();

    // name: commitment tx with one output untrimmed (minimum feerate)
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8001c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431100400473044022031a82b51bd014915fe68928d1abf4b9885353fb896cac10c3fdd88d7f9c7f2e00220716bda819641d2c63e65d3549b6120112e1aeaf1742eed94a471488e79e206b101473044022064901950be922e62cbe3f2ab93de2b99f37cff9fc473e73e394b27f88ef0731d02206d1dfa227527b4df44a07599289e207d6fd9cca60c0365682dcd3deaf739567e01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 9651181,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}

fn spec_ex_15_1() {
    let ex = get_example();

    // name: commitment tx with fee greater than funder amount
    let example_tx = s2tx("02000000000101bef67e4e2fb9ddeeb3461973cd4c62abb35050b1add772995b820b584a488489000000000038b02b8001c0c62d0000000000160014ccf1af2f2aabee14bb40fa3851ab2301de8431100400473044022031a82b51bd014915fe68928d1abf4b9885353fb896cac10c3fdd88d7f9c7f2e00220716bda819641d2c63e65d3549b6120112e1aeaf1742eed94a471488e79e206b101473044022064901950be922e62cbe3f2ab93de2b99f37cff9fc473e73e394b27f88ef0731d02206d1dfa227527b4df44a07599289e207d6fd9cca60c0365682dcd3deaf739567e01475221023da092f6980e58d2c037173180e9a465476026ee50f96695963e8efe436f54eb21030e9f7b623d2ccc7c9bd44d66d5ce21ce504c0acf6385a132cec6d3c39fa711c152ae3e195220");

    let mut commit_tx = CommitTx{
        funding_amount: ex.funding_amount_satoshi,

        local_feerate_per_kw: 9651936,
        dust_limit_satoshi: 546,

        to_local_msat: 6988000000,
        to_remote_msat: 3000000000,
        obscured_commit_number: ex.obscuring_factor ^ ex.commitment_number,

        local_htlc_pubkey: ex.localpubkey.clone(),
        remote_htlc_pubkey: ex.remotepubkey.clone(),

        local_revocation_pubkey: ex.local_revocation_pubkey.clone(),
        local_delayedpubkey: ex.local_delayedpubkey.clone(),
        local_delay: ex.local_delay as u64,

        remotepubkey: ex.remotepubkey.clone(),

        funding_tx_id: ex.funding_tx_id.clone(),
        funding_output_index: ex.funding_output_index as u32,

        htlcs: vec![],
    };

    for h in &ex.htlcs {
        commit_tx.htlcs.push(h.to_htlc());
    }

    let tx = commit_tx.get_tx();
    assert_tx_eq(&tx, &example_tx, true);
}


fn main() {
    println!("it works: create-funding-example");
    println!("Task is to create funding transaction and compare it hash to correct");
    // Known:
    // One funding input: 9fd6518132028825404a02cd02a1c4885317bf7e193674547f4b7cfec92d095b:1
    // One change output(index: 1): value:1240930000, lockScript: 0014177811f6ffea178e4fb96b4ed5e01a566cbd3fe2
    // Funding amount: 1000000
    // (Funder) Funding pubkey: 033df6f19a33d091d6196b20540de9ebfd8d704d8f05133add990eaffa946effbe
    // (Fundee) Funding pubkey: 03b6dc51a2d1b29e67a204656903a3b3ef42e45e3486eb8a506316acd36872422f
    // Funding output index:0
    // WE NEED TO OBTAIN: 47c632779ccfed3f3d36ba423eae871690e2d9270dde1df962b52a6c8be89336:0
    let pk1 = hex::decode("033df6f19a33d091d6196b20540de9ebfd8d704d8f05133add990eaffa946effbe").unwrap();
    let pk2 = hex::decode("03b6dc51a2d1b29e67a204656903a3b3ef42e45e3486eb8a506316acd36872422f").unwrap();

    let tx_in = TxIn {
        prev_hash: s2dh256("9fd6518132028825404a02cd02a1c4885317bf7e193674547f4b7cfec92d095b"),
        prev_index: 1,
        script_sig: Script::new(),
        sequence: 4294967295,
        witness: vec![vec![]]
    };

    let tx_out_funding = TxOut {
        value: 1000000,
        script_pubkey: new_2x2_wsh_lock_script(&pk1, &pk2),
    };

    let tx_out_change = TxOut {
        value: 1240930000,
        script_pubkey: s2script("0014177811f6ffea178e4fb96b4ed5e01a566cbd3fe2"),
    };
    assert_eq!(tx_out_funding.script_pubkey.data(), hex::decode("002085e247fa5331a24613863e86039d539cb21e012b966e014dd090f33eeb555760").unwrap());
    let tx = Transaction{
        version: 1,
        input: vec![tx_in],
        output: vec![tx_out_funding, tx_out_change],
        lock_time: 0
    };
    assert_eq!(tx.txid().be_hex_string(), "47c632779ccfed3f3d36ba423eae871690e2d9270dde1df962b52a6c8be89336");

    example_from_spec();

    spec_ex_1();
    spec_ex_2();
    spec_ex_1_1();
    spec_ex_2_1();
    spec_ex_3_1();
    spec_ex_4_1();
    spec_ex_5_1();
    spec_ex_6_1();
    spec_ex_7_1();
    spec_ex_8_1();
    spec_ex_9_1();
    spec_ex_10_1();
    spec_ex_11_1();
    spec_ex_12_1();
    spec_ex_13_1();
    spec_ex_14_1();
    spec_ex_15_1();
}