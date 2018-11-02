#[cfg(test)]
mod test_bolt0008;
mod async_stream;
mod serde;
mod cipher_state;
mod symmetric_state;

pub use self::async_stream::BrontideStream;

use std::{fmt, io, error, cell};
use secp256k1::{PublicKey, SecretKey, Error as EcdsaError};
use sha2::{Sha256, Digest};

use hex;
use hkdf;
use std;
use rand;

use tokio::timer::timeout;

use self::cipher_state::CipherState;
use self::symmetric_state::SymmetricState;

#[derive(Debug)]
pub enum HandshakeError {
    Io(io::Error),
    IoTimeout(timeout::Error<io::Error>),
    Crypto(EcdsaError),
    UnknownHandshakeVersion(String),
    NotInitializedYet,
}

impl error::Error for HandshakeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use self::HandshakeError::*;

        match self {
            &Io(ref e) => Some(e),
            &IoTimeout(ref e) => Some(e),
            &Crypto(ref e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::HandshakeError::*;

        match self {
            &Io(ref e) => write!(f, "io error: {}", e),
            &IoTimeout(ref e) => write!(f, "io timeout error: {}", e),
            &Crypto(ref e) => write!(f, "crypto error: {}", e),
            &UnknownHandshakeVersion(ref msg) => write!(f, "{}", msg),
            &NotInitializedYet => write!(f, "not initialized yet")
        }
    }
}

// PROTOCOL_NAME is the precise instantiation of the Noise protocol
// handshake at the center of Brontide. This value will be used as part
// of the prologue. If the initiator and responder aren't using the
// exact same string for this value, along with prologue of the Bitcoin
// network, then the initial handshake will fail.
static PROTOCOL_NAME: &'static str = "Noise_XK_secp256k1_ChaChaPoly_SHA256";

// MAC_SIZE is the length in bytes of the tags generated by poly1305.
const MAC_SIZE: usize = 16;

// LENGTH_HEADER_SIZE is the number of bytes used to prefix encode the
// length of a message payload.
const LENGTH_HEADER_SIZE: usize = 2;

// ERR_MAX_MESSAGE_LENGTH_EXCEEDED is returned a message to be written to
// the cipher session exceeds the maximum allowed message payload.
static ERR_MAX_MESSAGE_LENGTH_EXCEEDED: &'static str = "the generated payload exceeds the max allowed message length of (2^16)-1";

// ecdh performs an ECDH operation between public and private. The returned value is
// the sha256 of the compressed shared point.
fn ecdh(pk: &PublicKey, sk: &SecretKey) -> Result<[u8; 32], EcdsaError> {
    use secp256k1::Secp256k1;

    let mut pk_cloned = pk.clone();
    pk_cloned.mul_assign(&Secp256k1::new(), sk)?;

    let mut hasher = Sha256::default();
    hasher.input(&pk_cloned.serialize());
    let hash = hasher.result();

    let mut array: [u8; 32] = [0; 32];
    array.copy_from_slice(&hash);
    Ok(array)
}

// TODO(evg): we have changed encrypt/decrypt and encrypt_and_hash/decrypt_and_hash method signatures
// so it should be reflect in doc


// HandshakeState encapsulates the symmetricState and keeps track of all the
// public keys (static and ephemeral) for both sides during the handshake
// transcript. If the handshake completes successfully, then two instances of a
// cipherState are emitted: one to encrypt messages from initiator to
// responder, and the other for the opposite direction.
struct HandshakeState {
    symmetric_state: SymmetricState,

    initiator: bool,

    local_static:    SecretKey,
    // if None means not initialized
    local_ephemeral: Option<SecretKey>,

    remote_static:    PublicKey,
    // if None means not initialized
    remote_ephemeral: Option<PublicKey>,
}

impl fmt::Debug for HandshakeState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let remote_ephemeral_str = match self.remote_ephemeral {
            None => "None".to_owned(),
            Some(k) => hex::encode(&k.serialize()[..]),
        };

        write!(f, r#"
        symmetric_state: {:?}

        initiator: {:?}

        local_static:    {:?}
        local_ephemeral: {:?}

        remote_static:    {:?}
        remote_ephemeral: {:?}
        "#, self.symmetric_state, self.initiator,
               self.local_static, self.local_ephemeral,
               hex::encode(&self.remote_static.serialize()[..]),
               remote_ephemeral_str,
        )
    }
}

impl HandshakeState {
    // new returns a new instance of the handshake state initialized
    // with the prologue and protocol name. If this is the responder's handshake
    // state, then the remotePub can be nil.
    fn new(initiator: bool, prologue: &[u8],
           local_priv: SecretKey, remote_pub: PublicKey) -> Result<Self, EcdsaError> {
        use secp256k1::Secp256k1;

        let mut h = HandshakeState{
            symmetric_state: SymmetricState::new(),
            initiator,
            local_static:     local_priv,
            local_ephemeral:  None,
            remote_static:    remote_pub,
            remote_ephemeral: None,
        };

        // Set the current chaining key and handshake digest to the hash of the
        // protocol name, and additionally mix in the prologue. If either sides
        // disagree about the prologue or protocol name, then the handshake
        // will fail.
        h.symmetric_state.initialize_symmetric(PROTOCOL_NAME.as_bytes());
        h.symmetric_state.mix_hash(prologue);

        // In Noise_XK, then initiator should know the responder's static
        // public key, therefore we include the responder's static key in the
        // handshake digest. If the initiator gets this value wrong, then the
        // handshake will fail.
        if initiator {
            h.symmetric_state.mix_hash(&remote_pub.serialize())
        } else {
            let local_pub = PublicKey::from_secret_key(&Secp256k1::new(), &local_priv)?;
            h.symmetric_state.mix_hash(&local_pub.serialize())
        }

        Ok(h)
    }
}

pub struct Machine {
    send_cipher: cell::RefCell<CipherState>,
    recv_cipher: cell::RefCell<CipherState>,

    ephemeral_gen: fn() -> Result<SecretKey, EcdsaError>,

    handshake_state: HandshakeState,

    // a static buffer that we'll use to read in the
    // bytes of the next cipher text message. As all messages in the
    // protocol MUST be below 65KB plus our macSize, this will be
    // sufficient to buffer all messages from the socket when we need to
    // read the next one. Having a fixed buffer that's re-used also means
    // that we save on allocations as we don't need to create a new one
    // each time.
    message_buffer: cell::RefCell<[u8; std::u16::MAX as usize]>,
}

impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"
        send_cipher:     {:?}
        recv_cipher:     {:?}
        handshake_state: {:?}
        "#, self.send_cipher, self.recv_cipher, self.handshake_state,
        )
    }
}

impl Machine {
    // new creates a new instance of the brontide state-machine. If
    // the responder (listener) is creating the object, then the remotePub should
    // be nil. The handshake state within brontide is initialized using the ascii
    // string "lightning" as the prologue. The last parameter is a set of variadic
    // arguments for adding additional options to the brontide Machine
    // initialization.
    pub fn new<F>(initiator: bool, local_priv: SecretKey, remote_pub: PublicKey, options: &[F]) -> Result<Self, EcdsaError> where F: Fn(&mut Self) {
        use secp256k1::{Secp256k1, constants::SECRET_KEY_SIZE};

        let handshake = HandshakeState::new(initiator, "lightning".as_bytes(), local_priv, remote_pub)?;

        let mut m = Machine {
            send_cipher: cell::RefCell::new(CipherState::new()),
            recv_cipher: cell::RefCell::new(CipherState::new()),
            // With the initial base machine created, we'll assign our default
            // version of the ephemeral key generator.
            ephemeral_gen: || {
                let sk_bytes: [u8; SECRET_KEY_SIZE] = rand::random();
                let sk = SecretKey::from_slice(&Secp256k1::new(), &sk_bytes)?;
                Ok(sk)
            },
            handshake_state: handshake,
            message_buffer: cell::RefCell::new([0; std::u16::MAX as usize]),
        };

        // With the default options established, we'll now process all the
        // options passed in as parameters.
        for option in options {
            option(&mut m)
        }

        Ok(m)
    }
}

// HANDSHAKE_VERSION is the expected version of the brontide handshake.
// Any messages that carry a different version will cause the handshake
// to abort immediately.
#[repr(u8)]
#[derive(Eq, PartialEq)]
enum HandshakeVersion {
    _0 = 0,
}

// ACT_ONE_SIZE is the size of the packet sent from initiator to
// responder in ActOne. The packet consists of a handshake version, an
// ephemeral key in compressed format, and a 16-byte poly1305 tag.
//
// 1 + 33 + 16
struct ActOne {
    bytes: [u8; 1 + 33 + MAC_SIZE],
}

impl Default for ActOne {
    fn default() -> Self {
        ActOne {
            bytes: [0; Self::SIZE],
        }
    }
}

impl AsRef<[u8]> for ActOne {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..]
    }
}

impl AsMut<[u8]> for ActOne {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.bytes[..]
    }
}

impl ActOne {
    const SIZE: usize = 1 + 33 + MAC_SIZE;

    fn new(version: HandshakeVersion, key: [u8; 33], tag: [u8; MAC_SIZE]) -> Self {
        let mut s = ActOne {
            bytes: [0; Self::SIZE],
        };
        s.bytes[0] = version as _;
        s.bytes[1..34].copy_from_slice(&key);
        s.bytes[34..].copy_from_slice(&tag);
        s
    }

    fn version(&self) -> Result<HandshakeVersion, ()> {
        match self.bytes[0] {
            0 => Ok(HandshakeVersion::_0),
            _ => Err(())
        }
    }

    fn key(&self) -> Result<PublicKey, EcdsaError> {
        use secp256k1::Secp256k1;

        PublicKey::from_slice(&Secp256k1::new(), &self.bytes[1..34])
    }

    fn tag(&self) -> [u8; MAC_SIZE] {
        let mut v = [0; MAC_SIZE];
        v.copy_from_slice(&self.bytes[34..]);
        v
    }
}

// ACT_TWO_SIZE is the size the packet sent from responder to initiator
// in ActTwo. The packet consists of a handshake version, an ephemeral
// key in compressed format and a 16-byte poly1305 tag.
//
// 1 + 33 + 16
type ActTwo = ActOne;

// ACT_THREE_SIZE is the size of the packet sent from initiator to
// responder in ActThree. The packet consists of a handshake version,
// the initiators static key encrypted with strong forward secrecy and
// a 16-byte poly1035
// tag.
//
// 1 + 33 + 16 + 16
struct ActThree {
    bytes: [u8; 1 + 33 + 16 + 16],
}

impl Default for ActThree {
    fn default() -> Self {
        ActThree {
            bytes: [0; Self::SIZE],
        }
    }
}

impl AsRef<[u8]> for ActThree {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..]
    }
}

impl AsMut<[u8]> for ActThree {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.bytes[..]
    }
}

impl ActThree {
    const SIZE: usize = 1 + 33 + 2 * MAC_SIZE;

    fn new(version: HandshakeVersion, key: Vec<u8>, tag_first: [u8; MAC_SIZE], tag_second: [u8; MAC_SIZE]) -> Self {
        let mut s = ActThree {
            bytes: [0; Self::SIZE],
        };
        s.bytes[0] = version as _;
        s.bytes[1..34].copy_from_slice(&key);
        s.bytes[34..50].copy_from_slice(&tag_first);
        s.bytes[50..].copy_from_slice(&tag_second);
        s
    }

    fn version(&self) -> Result<HandshakeVersion, ()> {
        match self.bytes[0] {
            0 => Ok(HandshakeVersion::_0),
            _ => Err(())
        }
    }

    fn key(&self) -> &[u8] {
        &self.bytes[1..34]
    }

    fn tag_first(&self) -> [u8; MAC_SIZE] {
        let mut v = [0; MAC_SIZE];
        v.copy_from_slice(&self.bytes[34..50]);
        v
    }

    fn tag_second(&self) -> [u8; MAC_SIZE] {
        let mut v = [0; MAC_SIZE];
        v.copy_from_slice(&self.bytes[50..]);
        v
    }
}

impl Machine {
    // gen_act_one generates the initial packet (act one) to be sent from initiator
    // to responder. During act one the initiator generates a fresh ephemeral key,
    // hashes it into the handshake digest, and performs an ECDH between this key
    // and the responder's static key. Future payloads are encrypted with a key
    // derived from this result.
    //
    //    -> e, es
    fn gen_act_one(&mut self) -> Result<ActOne, HandshakeError> {
        use secp256k1::Secp256k1;

        // e
        let local_ephemeral_priv = (self.ephemeral_gen)()
            .map_err(HandshakeError::Crypto)?;
        self.handshake_state.local_ephemeral = Some(local_ephemeral_priv);

        let local_ephemeral_pub = PublicKey::from_secret_key(&Secp256k1::new(), &local_ephemeral_priv)
            .map_err(HandshakeError::Crypto)?;
        let ephemeral = local_ephemeral_pub.serialize();
        self.handshake_state.symmetric_state.mix_hash(&ephemeral);

        // es
        let s = ecdh(&self.handshake_state.remote_static, &local_ephemeral_priv)
            .map_err(HandshakeError::Crypto)?;
        self.handshake_state.symmetric_state.mix_key(&s);

        let auth_payload = self.handshake_state.symmetric_state
            .encrypt_and_hash(&[], &mut Vec::new())
            .map_err(HandshakeError::Io)?;

        Ok(ActOne::new(HandshakeVersion::_0, ephemeral, auth_payload))
    }

    // recv_act_one processes the act one packet sent by the initiator. The responder
    // executes the mirrored actions to that of the initiator extending the
    // handshake digest and deriving a new shared secret based on an ECDH with the
    // initiator's ephemeral key and responder's static key.
    fn recv_act_one(&mut self, act_one: ActOne) -> Result<(), HandshakeError> {
        // If the handshake version is unknown, then the handshake fails
        // immediately.
        if let Err(()) = act_one.version() {
            let msg = format!("Act One: invalid handshake version: {}", act_one.bytes[0]);
            return Err(HandshakeError::UnknownHandshakeVersion(msg))
        }

        // e
        let remote_ephemeral = act_one.key()
            .map_err(HandshakeError::Crypto)?;
        self.handshake_state.remote_ephemeral = Some(remote_ephemeral);
        self.handshake_state.symmetric_state.mix_hash(&remote_ephemeral.serialize());

        // es
        let s = ecdh(&remote_ephemeral, &self.handshake_state.local_static)
            .map_err(HandshakeError::Crypto)?;
        self.handshake_state.symmetric_state.mix_key(&s);

        // If the initiator doesn't know our static key, then this operation
        // will fail.
        self.handshake_state.symmetric_state
            .decrypt_and_hash(&[], act_one.tag())
            .map_err(HandshakeError::Io)?;

        Ok(())
    }

    // gen_act_two generates the second packet (act two) to be sent from the
    // responder to the initiator. The packet for act two is identify to that of
    // act one, but then results in a different ECDH operation between the
    // initiator's and responder's ephemeral keys.
    //
    //    <- e, ee
    fn gen_act_two(&mut self) -> Result<ActTwo, HandshakeError> {
        use secp256k1::Secp256k1;

        // e
        let local_ephemeral_priv = (self.ephemeral_gen)().map_err(HandshakeError::Crypto)?;
        self.handshake_state.local_ephemeral = Some(local_ephemeral_priv);

        let local_ephemeral_pub = PublicKey::from_secret_key(
            &Secp256k1::new(), &local_ephemeral_priv).map_err(HandshakeError::Crypto)?;
        let ephemeral = local_ephemeral_pub.serialize();
        self.handshake_state.symmetric_state.mix_hash(&ephemeral);

        // ee
        let s = ecdh(&self.handshake_state.remote_ephemeral.ok_or(HandshakeError::NotInitializedYet)?, &local_ephemeral_priv)
            .map_err(HandshakeError::Crypto)?;
        self.handshake_state.symmetric_state.mix_key(&s);

        let auth_payload = self.handshake_state.symmetric_state
            .encrypt_and_hash(&[], &mut Vec::new())
            .map_err(HandshakeError::Io)?;

        Ok(ActTwo::new(HandshakeVersion::_0, ephemeral, auth_payload))
    }

    // recv_act_two processes the second packet (act two) sent from the responder to
    // the initiator. A successful processing of this packet authenticates the
    // initiator to the responder.
    fn recv_act_two(&mut self, act_two: ActTwo) -> Result<(), HandshakeError> {
        // If the handshake version is unknown, then the handshake fails
        // immediately.
        if let Err(()) = act_two.version() {
            let msg = format!("Act Two: invalid handshake version: {}", act_two.bytes[0]);
            return Err(HandshakeError::UnknownHandshakeVersion(msg))
        }

        // e
        let remote_ephemeral = act_two.key()
            .map_err(HandshakeError::Crypto)?;
        self.handshake_state.remote_ephemeral = Some(remote_ephemeral);
        self.handshake_state.symmetric_state.mix_hash(&remote_ephemeral.serialize());

        // ee
        let s = ecdh(&remote_ephemeral, &self.handshake_state.local_ephemeral.ok_or(HandshakeError::NotInitializedYet)?)
            .map_err(HandshakeError::Crypto)?;
        self.handshake_state.symmetric_state.mix_key(&s);

        self.handshake_state.symmetric_state
            .decrypt_and_hash(&mut Vec::new(), act_two.tag())
            .map_err(HandshakeError::Io)?;
        Ok(())
    }

    // gen_act_three creates the final (act three) packet of the handshake. Act three
    // is to be sent from the initiator to the responder. The purpose of act three
    // is to transmit the initiator's public key under strong forward secrecy to
    // the responder. This act also includes the final ECDH operation which yields
    // the final session.
    //
    //    -> s, se
    fn gen_act_three(&mut self) -> Result<ActThree, HandshakeError> {
        use secp256k1::{Secp256k1, constants::PUBLIC_KEY_SIZE};

        let local_static_pub = PublicKey::from_secret_key(&Secp256k1::new(), &self.handshake_state.local_static)
            .map_err(HandshakeError::Crypto)?;
        let our_pubkey = local_static_pub.serialize();
        let mut ciphertext = Vec::with_capacity(PUBLIC_KEY_SIZE);
        let tag = self.handshake_state.symmetric_state
            .encrypt_and_hash(&our_pubkey, &mut ciphertext)
            .map_err(HandshakeError::Io)?;

        let s = ecdh(&self.handshake_state.remote_ephemeral.ok_or(HandshakeError::NotInitializedYet)?, &self.handshake_state.local_static)
            .map_err(HandshakeError::Crypto)?;
        self.handshake_state.symmetric_state.mix_key(&s);

        let auth_payload = self.handshake_state.symmetric_state
            .encrypt_and_hash(&[], &mut Vec::new())
            .map_err(HandshakeError::Io)?;

        let act_three = ActThree::new(HandshakeVersion::_0, ciphertext, tag, auth_payload);

        // With the final ECDH operation complete, derive the session sending
        // and receiving keys.
        self.split();

        Ok(act_three)
    }

    // recv_act_three processes the final act (act three) sent from the initiator to
    // the responder. After processing this act, the responder learns of the
    // initiator's static public key. Decryption of the static key serves to
    // authenticate the initiator to the responder.
    fn recv_act_three(&mut self, act_three: ActThree) -> Result<(), HandshakeError> {
        use secp256k1::Secp256k1;

        // If the handshake version is unknown, then the handshake fails
        // immediately.
        if let Err(()) = act_three.version() {
            let msg = format!("Act Three: invalid handshake version: {}", act_three.bytes[0]);
            return Err(HandshakeError::UnknownHandshakeVersion(msg))
        }

        // s
        let remote_pub = self.handshake_state.symmetric_state.decrypt_and_hash(act_three.key(), act_three.tag_first())
            .map_err(HandshakeError::Io)?;
        self.handshake_state.remote_static = PublicKey::from_slice(&Secp256k1::new(), &remote_pub)
            .map_err(HandshakeError::Crypto)?;

        // se
        let se = ecdh(&self.handshake_state.remote_static, &self.handshake_state.local_ephemeral.ok_or(HandshakeError::NotInitializedYet)?)
            .map_err(HandshakeError::Crypto)?;
        self.handshake_state.symmetric_state.mix_key(&se);

        self.handshake_state.symmetric_state
            .decrypt_and_hash(&[], act_three.tag_second())
            .map_err(HandshakeError::Io)?;

        // With the final ECDH operation complete, derive the session sending
        // and receiving keys.
        self.split();

        Ok(())
    }

    // split is the final wrap-up act to be executed at the end of a successful
    // three act handshake. This function creates two internal cipherState
    // instances: one which is used to encrypt messages from the initiator to the
    // responder, and another which is used to encrypt message for the opposite
    // direction.
    fn split(&mut self) {
        let mut send_key: [u8; 32] = [0; 32];
        let mut recv_key: [u8; 32] = [0; 32];

        let hkdf = hkdf::Hkdf::<Sha256>::extract(Some(&self.handshake_state.symmetric_state.chaining_key), &[]);
        let okm = hkdf.expand(&[], 64);

        // If we're the initiator the first 32 bytes are used to encrypt our
        // messages and the second 32-bytes to decrypt their messages. For the
        // responder the opposite is true.
        if self.handshake_state.initiator {
            send_key.copy_from_slice(&okm.as_slice()[..32]);
            self.send_cipher.borrow_mut().initialize_key_with_salt(self.handshake_state.symmetric_state.chaining_key, send_key);

            recv_key.copy_from_slice(&okm.as_slice()[32..]);
            self.recv_cipher.borrow_mut().initialize_key_with_salt(self.handshake_state.symmetric_state.chaining_key, recv_key);
        } else {
            recv_key.copy_from_slice(&okm.as_slice()[..32]);
            self.recv_cipher.borrow_mut().initialize_key_with_salt(self.handshake_state.symmetric_state.chaining_key, recv_key);

            send_key.copy_from_slice(&okm.as_slice()[32..]);
            self.send_cipher.borrow_mut().initialize_key_with_salt(self.handshake_state.symmetric_state.chaining_key, send_key);
        }
    }
}
