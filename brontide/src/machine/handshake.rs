use std::{io, fmt, error, cell};
use tokio::timer::timeout;
use secp256k1::{SecretKey, PublicKey, Error as EcdsaError};
use super::cipher_state::CipherState;
use super::symmetric_state::{SymmetricState, MAC_SIZE};

// ecdh performs an ECDH operation between public and private. The returned value is
// the sha256 of the compressed shared point.
fn ecdh(pk: &PublicKey, sk: &SecretKey) -> Result<[u8; 32], EcdsaError> {
    use secp256k1::Secp256k1;
    use sha2::{Sha256, Digest};

    let mut pk_cloned = pk.clone();
    pk_cloned.mul_assign(&Secp256k1::new(), sk)?;

    let mut hasher = Sha256::default();
    hasher.input(&pk_cloned.serialize());
    let hash = hasher.result();

    let mut array: [u8; 32] = [0; 32];
    array.copy_from_slice(&hash);
    Ok(array)
}

#[derive(Debug)]
pub enum HandshakeError {
    Io(io::Error),
    IoTimeout(timeout::Error<io::Error>),
    Crypto(EcdsaError),
    UnknownHandshakeVersion(String),
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
        }
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
pub struct ActOne {
    pub bytes: [u8; 1 + 33 + MAC_SIZE],
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
            _ => Err(()),
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
pub type ActTwo = ActOne;

// ACT_THREE_SIZE is the size of the packet sent from initiator to
// responder in ActThree. The packet consists of a handshake version,
// the initiators static key encrypted with strong forward secrecy and
// a 16-byte poly1035
// tag.
//
// 1 + 33 + 16 + 16
pub struct ActThree {
    pub bytes: [u8; 1 + 33 + 16 + 16],
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

    fn new(
        version: HandshakeVersion,
        key: Vec<u8>,
        tag_first: [u8; MAC_SIZE],
        tag_second: [u8; MAC_SIZE],
    ) -> Self {
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
            _ => Err(()),
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

// PROTOCOL_NAME is the precise instantiation of the Noise protocol
// handshake at the center of Brontide. This value will be used as part
// of the prologue. If the initiator and responder aren't using the
// exact same string for this value, along with prologue of the Bitcoin
// network, then the initial handshake will fail.
static PROTOCOL_NAME: &'static str = "Noise_XK_secp256k1_ChaChaPoly_SHA256";

pub struct HandshakeNew {
    symmetric_state: SymmetricState,
    local_static: SecretKey,
    remote_static: PublicKey,
    pub ephemeral_gen: fn() -> Result<SecretKey, EcdsaError>,
}

impl HandshakeNew {
    pub fn new(
        initiator: bool,
        local_secret: SecretKey,
        remote_public: PublicKey,
    ) -> Result<Self, EcdsaError> {
        use secp256k1::{Secp256k1, constants::SECRET_KEY_SIZE};

        let mut symmetric_state = SymmetricState::new(PROTOCOL_NAME);
        symmetric_state.mix_hash("lightning".as_bytes());
        if initiator {
            symmetric_state.mix_hash(&remote_public.serialize());
        } else {
            let local_pub = PublicKey::from_secret_key(&Secp256k1::new(), &local_secret)?;
            symmetric_state.mix_hash(&local_pub.serialize());
        }

        Ok(HandshakeNew {
            symmetric_state: symmetric_state,
            local_static: local_secret,
            remote_static: remote_public,
            ephemeral_gen: || {
                let sk: [u8; SECRET_KEY_SIZE] = rand::random();
                SecretKey::from_slice(&Secp256k1::new(), &sk)
            },
        })
    }

    // gen_act_one generates the initial packet (act one) to be sent from initiator
    // to responder. During act one the initiator generates a fresh ephemeral key,
    // hashes it into the handshake digest, and performs an ECDH between this key
    // and the responder's static key. Future payloads are encrypted with a key
    // derived from this result.
    //
    //    -> e, es
    pub fn gen_act_one(mut self) -> Result<(ActOne, HandshakeInitiatorActOne), HandshakeError> {
        use secp256k1::Secp256k1;

        // e
        let local_ephemeral = (self.ephemeral_gen)().map_err(HandshakeError::Crypto)?;

        let local_ephemeral_pub = PublicKey::from_secret_key(&Secp256k1::new(), &local_ephemeral)
            .map_err(HandshakeError::Crypto)?;
        let ephemeral = local_ephemeral_pub.serialize();
        self.symmetric_state.mix_hash(&ephemeral);

        // es
        let s = ecdh(&self.remote_static, &local_ephemeral).map_err(HandshakeError::Crypto)?;
        self.symmetric_state.mix_key(&s);

        let auth_payload = self
            .symmetric_state
            .encrypt_and_hash(&[], &mut Vec::new())
            .map_err(HandshakeError::Io)?;

        let act_one = ActOne::new(HandshakeVersion::_0, ephemeral, auth_payload);
        let handshake_act_one = HandshakeInitiatorActOne {
            base: self,
            local_ephemeral: local_ephemeral,
        };
        Ok((act_one, handshake_act_one))
    }

    // recv_act_one processes the act one packet sent by the initiator. The responder
    // executes the mirrored actions to that of the initiator extending the
    // handshake digest and deriving a new shared secret based on an ECDH with the
    // initiator's ephemeral key and responder's static key.
    pub fn recv_act_one(mut self, act_one: ActOne) -> Result<HandshakeActOne, HandshakeError> {
        // If the handshake version is unknown, then the handshake fails
        // immediately.
        if let Err(()) = act_one.version() {
            let msg = format!("Act One: invalid handshake version: {}", act_one.bytes[0]);
            return Err(HandshakeError::UnknownHandshakeVersion(msg));
        }

        // e
        let remote_ephemeral = act_one.key().map_err(HandshakeError::Crypto)?;
        self.symmetric_state.mix_hash(&remote_ephemeral.serialize());

        // es
        let s = ecdh(&remote_ephemeral, &self.local_static).map_err(HandshakeError::Crypto)?;
        self.symmetric_state.mix_key(&s);

        // If the initiator doesn't know our static key, then this operation
        // will fail.
        self.symmetric_state
            .decrypt_and_hash(&[], act_one.tag())
            .map_err(HandshakeError::Io)?;

        Ok(HandshakeActOne {
            base: self,
            remote_ephemeral: remote_ephemeral,
        })
    }

    #[cfg(test)]
    pub fn handshake_digest(&self) -> [u8; 32] {
        self.symmetric_state.handshake_digest()
    }
}

pub struct HandshakeActOne {
    base: HandshakeNew,
    remote_ephemeral: PublicKey,
}

impl HandshakeActOne {
    // gen_act_two generates the second packet (act two) to be sent from the
    // responder to the initiator. The packet for act two is identify to that of
    // act one, but then results in a different ECDH operation between the
    // initiator's and responder's ephemeral keys.
    //
    //    <- e, ee
    pub fn gen_act_two(mut self) -> Result<(ActTwo, Handshake), HandshakeError> {
        use secp256k1::Secp256k1;

        // e
        let local_ephemeral = (self.base.ephemeral_gen)().map_err(HandshakeError::Crypto)?;

        let local_ephemeral_pub = PublicKey::from_secret_key(&Secp256k1::new(), &local_ephemeral)
            .map_err(HandshakeError::Crypto)?;
        let ephemeral = local_ephemeral_pub.serialize();
        self.base.symmetric_state.mix_hash(&ephemeral);

        // ee
        let s = ecdh(&self.remote_ephemeral, &local_ephemeral).map_err(HandshakeError::Crypto)?;
        self.base.symmetric_state.mix_key(&s);

        let auth_payload = self
            .base
            .symmetric_state
            .encrypt_and_hash(&[], &mut Vec::new())
            .map_err(HandshakeError::Io)?;

        let act_two = ActTwo::new(HandshakeVersion::_0, ephemeral, auth_payload);
        let handshake = Handshake {
            base: self.base,
            local_ephemeral: local_ephemeral,
            remote_ephemeral: self.remote_ephemeral,
        };
        Ok((act_two, handshake))
    }
}

pub struct HandshakeInitiatorActOne {
    base: HandshakeNew,
    local_ephemeral: SecretKey,
}

impl HandshakeInitiatorActOne {
    // recv_act_two processes the second packet (act two) sent from the responder to
    // the initiator. A successful processing of this packet authenticates the
    // initiator to the responder.
    pub fn recv_act_two(mut self, act_two: ActTwo) -> Result<Handshake, HandshakeError> {
        // If the handshake version is unknown, then the handshake fails
        // immediately.
        if let Err(()) = act_two.version() {
            let msg = format!("Act Two: invalid handshake version: {}", act_two.bytes[0]);
            return Err(HandshakeError::UnknownHandshakeVersion(msg));
        }

        // e
        let remote_ephemeral = act_two.key().map_err(HandshakeError::Crypto)?;
        self.base
            .symmetric_state
            .mix_hash(&remote_ephemeral.serialize());

        // ee
        let s = ecdh(&remote_ephemeral, &self.local_ephemeral).map_err(HandshakeError::Crypto)?;
        self.base.symmetric_state.mix_key(&s);

        self.base
            .symmetric_state
            .decrypt_and_hash(&mut Vec::new(), act_two.tag())
            .map_err(HandshakeError::Io)?;

        Ok(Handshake {
            base: self.base,
            local_ephemeral: self.local_ephemeral,
            remote_ephemeral: remote_ephemeral,
        })
    }
}

pub struct Handshake {
    base: HandshakeNew,
    local_ephemeral: SecretKey,
    remote_ephemeral: PublicKey,
}

impl Handshake {
    // gen_act_three creates the final (act three) packet of the handshake. Act three
    // is to be sent from the initiator to the responder. The purpose of act three
    // is to transmit the initiator's public key under strong forward secrecy to
    // the responder. This act also includes the final ECDH operation which yields
    // the final session.
    //
    //    -> s, se
    pub fn gen_act_three(mut self) -> Result<(ActThree, Machine), HandshakeError> {
        use secp256k1::{Secp256k1, constants::PUBLIC_KEY_SIZE};

        let local_static_pub =
            PublicKey::from_secret_key(&Secp256k1::new(), &self.base.local_static)
                .map_err(HandshakeError::Crypto)?;
        let our_pubkey = local_static_pub.serialize();
        let mut cipher_text = Vec::with_capacity(PUBLIC_KEY_SIZE);
        let tag = self
            .base
            .symmetric_state
            .encrypt_and_hash(&our_pubkey, &mut cipher_text)
            .map_err(HandshakeError::Io)?;

        let s = ecdh(&self.remote_ephemeral, &self.base.local_static)
            .map_err(HandshakeError::Crypto)?;
        self.base.symmetric_state.mix_key(&s);

        let auth_payload = self
            .base
            .symmetric_state
            .encrypt_and_hash(&[], &mut Vec::new())
            .map_err(HandshakeError::Io)?;

        let act_three = ActThree::new(HandshakeVersion::_0, cipher_text, tag, auth_payload);

        // With the final ECDH operation complete, derive the session sending
        // and receiving keys.
        Ok((act_three, self.split(false)))
    }

    // recv_act_three processes the final act (act three) sent from the initiator to
    // the responder. After processing this act, the responder learns of the
    // initiator's static public key. Decryption of the static key serves to
    // authenticate the initiator to the responder.
    pub fn recv_act_three(mut self, act_three: ActThree) -> Result<Machine, HandshakeError> {
        use secp256k1::Secp256k1;

        // If the handshake version is unknown, then the handshake fails
        // immediately.
        if let Err(()) = act_three.version() {
            let msg = format!(
                "Act Three: invalid handshake version: {}",
                act_three.bytes[0]
            );
            return Err(HandshakeError::UnknownHandshakeVersion(msg));
        }

        // s
        let remote_pub = self
            .base
            .symmetric_state
            .decrypt_and_hash(act_three.key(), act_three.tag_first())
            .map_err(HandshakeError::Io)?;
        self.base.remote_static = PublicKey::from_slice(&Secp256k1::new(), &remote_pub)
            .map_err(HandshakeError::Crypto)?;

        // se
        let se = ecdh(&self.base.remote_static, &self.local_ephemeral)
            .map_err(HandshakeError::Crypto)?;
        self.base.symmetric_state.mix_key(&se);

        self.base
            .symmetric_state
            .decrypt_and_hash(&[], act_three.tag_second())
            .map_err(HandshakeError::Io)?;

        // With the final ECDH operation complete, derive the session sending
        // and receiving keys.
        // swap them
        Ok(self.split(true))
    }

    fn split(self, swap: bool) -> Machine {
        #[cfg(test)]
        let chaining_key = self.base.symmetric_state.chaining_key();
        let (send, recv) = self.base.symmetric_state.into_pair();
        let (send, recv) = if swap { (recv, send) } else { (send, recv) };
        Machine {
            send_cipher: send,
            recv_cipher: recv,
            remote_static: self.base.remote_static,
            #[cfg(test)]
            chaining_key: chaining_key,
            message_buffer: cell::RefCell::new([0; std::u16::MAX as usize]),
        }
    }
}

use bytes::BytesMut;
use wire::{BinarySD, WireError};
use serde::{Serialize, de::DeserializeOwned};

pub struct Machine {
    send_cipher: CipherState,
    recv_cipher: CipherState,
    remote_static: PublicKey,
    #[cfg(test)]
    chaining_key: [u8; 32],
    message_buffer: cell::RefCell<[u8; std::u16::MAX as usize]>,
}

impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#"
        send_cipher:     {:?}
        recv_cipher:     {:?}
        remote_static:   {:?}
        "#,
            self.send_cipher, self.recv_cipher, self.remote_static,
        )
    }
}

// LENGTH_HEADER_SIZE is the number of bytes used to prefix encode the
// length of a message payload.
const LENGTH_HEADER_SIZE: usize = 2;

// ERR_MAX_MESSAGE_LENGTH_EXCEEDED is returned a message to be written to
// the cipher session exceeds the maximum allowed message payload.
static ERR_MAX_MESSAGE_LENGTH_EXCEEDED: &'static str =
    "the generated payload exceeds the max allowed message length of (2^16)-1";

impl Machine {
    pub fn remote_static(&self) -> &PublicKey {
        &self.remote_static
    }

    pub fn write<T>(&mut self, item: T, dst: &mut BytesMut) -> Result<(), WireError>
    where
        T: Serialize,
    {
        use bytes::BufMut;

        let length = {
            let mut buffer = self.message_buffer.borrow_mut();
            let mut cursor = io::Cursor::new(buffer.as_mut());
            BinarySD::serialize(&mut cursor, &item)?;
            cursor.position() as usize
        };

        if length > std::u16::MAX as usize {
            panic!(ERR_MAX_MESSAGE_LENGTH_EXCEEDED);
        }

        let mut length_buffer = [0; LENGTH_HEADER_SIZE];
        BinarySD::serialize(&mut length_buffer.as_mut(), &(length as u16))?;

        dst.reserve(length + LENGTH_HEADER_SIZE + MAC_SIZE * 2);

        let tag = self
            .send_cipher
            .encrypt(&[], &mut dst.writer(), &length_buffer[..])?;
        dst.put_slice(&tag[..]);

        let tag = self.send_cipher.encrypt(
            &[],
            &mut dst.writer(),
            &self.message_buffer.borrow()[..length],
        )?;
        dst.put_slice(&tag[..]);

        Ok(())
    }

    pub fn read<T>(&mut self, src: &mut BytesMut) -> Result<Option<T>, WireError>
    where
        T: DeserializeOwned,
    {
        use chacha20_poly1305_aead::DecryptError;
        use serde::ser::Error;

        if src.len() < LENGTH_HEADER_SIZE + MAC_SIZE {
            Ok(None)
        } else {
            let tag = |src: &mut BytesMut| {
                let tag_bytes = src.split_to(MAC_SIZE);
                let mut tag = [0; MAC_SIZE];
                tag.copy_from_slice(tag_bytes.as_ref());
                tag
            };

            let length = {
                let cipher = src.split_to(LENGTH_HEADER_SIZE);
                let tag = tag(src);

                let mut plain = [0; LENGTH_HEADER_SIZE];
                self.recv_cipher
                    .decrypt(&[], &mut plain.as_mut(), cipher.as_ref(), tag)
                    .map_err(|e| match e {
                        DecryptError::IoError(e) => WireError::from(e),
                        DecryptError::TagMismatch => WireError::custom("tag"),
                    })?;

                let length: u16 = BinarySD::deserialize(&plain[..])?;
                length as usize
            };

            if src.len() < length + MAC_SIZE {
                Ok(None)
            } else {
                let cipher = src.split_to(length);
                let tag = tag(src);

                self.recv_cipher
                    .decrypt(
                        &[],
                        &mut self.message_buffer.borrow_mut().as_mut(),
                        cipher.as_ref(),
                        tag,
                    ).map_err(|e| match e {
                        DecryptError::IoError(e) => WireError::from(e),
                        DecryptError::TagMismatch => WireError::custom("tag"),
                    })?;

                BinarySD::deserialize(self.message_buffer.borrow().as_ref()).map(Some)
            }
        }
    }

    #[cfg(test)]
    pub fn send_cipher_key(&self) -> [u8; 32] {
        self.send_cipher.secret_key()
    }

    #[cfg(test)]
    pub fn recv_cipher_key(&self) -> [u8; 32] {
        self.recv_cipher.secret_key()
    }

    #[cfg(test)]
    pub fn chaining_key(&self) -> [u8; 32] {
        self.chaining_key.clone()
    }
}
