use std::{io, fmt, error, sync::{RwLock, Arc}};

use dependencies::rand;
use dependencies::chacha20_poly1305_aead;
use dependencies::hex;
use dependencies::secp256k1;
use dependencies::tokio;
use dependencies::bytes;

use tokio::timer::timeout;
use secp256k1::{Secp256k1, SignOnly, VerifyOnly, SecretKey, PublicKey, Error as EcdsaError};
use super::cipher_state::CipherState;
use super::symmetric_state::{SymmetricState, MAC_SIZE};

#[derive(Debug)]
pub enum HandshakeError {
    Io(io::Error, String),
    IoTimeout(timeout::Error<io::Error>, String),
    Crypto(EcdsaError, String),
    UnknownHandshakeVersion(String),
}

impl error::Error for HandshakeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use self::HandshakeError::*;

        match self {
            &Io(ref e, _) => Some(e),
            &IoTimeout(ref e, _) => Some(e),
            &Crypto(ref e, _) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::HandshakeError::*;

        match self {
            &Io(ref e, ref desc) => write!(f, "io error, {}: {}", desc, e),
            &IoTimeout(ref e, ref desc) => write!(f, "io timeout error, {}: {}", desc, e),
            &Crypto(ref e, ref desc) => write!(f, "crypto error, {}: {}", desc, e),
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
    bytes: [u8; 1 + 33 + MAC_SIZE],
}

impl fmt::Debug for ActOne {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ActOne[ {} ]", hex::encode(&self.bytes[..]))
    }
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
        PublicKey::from_slice(&self.bytes[1..34])
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
pub struct ActTwo(ActOne);

impl Default for ActTwo {
    fn default() -> Self {
        dbg!("ActTwo::default()");
        ActTwo(Default::default())
    }
}

impl AsRef<[u8]> for ActTwo {
    fn as_ref(&self) -> &[u8] {
        &self.0.bytes[..]
    }
}

impl AsMut<[u8]> for ActTwo {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0.bytes[..]
    }
}

// ACT_THREE_SIZE is the size of the packet sent from initiator to
// responder in ActThree. The packet consists of a handshake version,
// the initiators static key encrypted with strong forward secrecy and
// a 16-byte poly1035
// tag.
//
// 1 + 33 + 16 + 16
pub struct ActThree {
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

    fn new(
        version: HandshakeVersion,
        key: Vec<u8>,
        tag_first: [u8; MAC_SIZE],
        tag_second: [u8; MAC_SIZE],
    ) -> Self {
        dbg!("ActThree::new()");
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

pub struct HandshakeIn {
    contexts: (Secp256k1<SignOnly>, Secp256k1<VerifyOnly>),
    symmetric_state: SymmetricState,
    local_static: SecretKey,
    pub ephemeral_gen: fn() -> SecretKey,
}

impl HandshakeIn {
    pub fn new(local_secret: SecretKey) -> Result<Self, EcdsaError> {
        let contexts = (Secp256k1::signing_only(), Secp256k1::verification_only());

        let mut symmetric_state = SymmetricState::new(PROTOCOL_NAME);
        symmetric_state.mix_hash("lightning".as_bytes());
        let local_pub = PublicKey::from_secret_key(&contexts.0, &local_secret);
        symmetric_state.mix_hash(&local_pub.serialize());

        Ok(HandshakeIn {
            contexts: contexts,
            symmetric_state: symmetric_state,
            local_static: local_secret,
            ephemeral_gen: || {
                SecretKey::new(&mut rand::thread_rng())
            },
        })
    }

    // receive_act_one processes the act one packet sent by the initiator. The responder
    // executes the mirrored actions to that of the initiator extending the
    // handshake digest and deriving a new shared secret based on an ECDH with the
    // initiator's ephemeral key and responder's static key.
    pub fn receive_act_one(mut self, act_one: ActOne) -> Result<HandshakeInActOne, HandshakeError> {
        use common_types::ac::SecretKey;
        let contexts = &self.contexts;

        // If the handshake version is unknown, then the handshake fails
        // immediately.
        if let Err(()) = act_one.version() {
            let msg = format!("Act One: invalid handshake version: {}", act_one.bytes[0]);
            return Err(HandshakeError::UnknownHandshakeVersion(msg));
        }

        // e
        let remote_ephemeral = act_one.key().map_err(|err| {
            HandshakeError::Crypto(err, "cannot create ephemeral public key from bytes".to_owned())
        })?;
        self.symmetric_state.mix_hash(&remote_ephemeral.serialize());

        // es
        let s = self.local_static
            .dh(&contexts.1, &remote_ephemeral)
            .map_err(|err| {
                HandshakeError::Crypto(err, "cannot calculate Diffie-Hellman (es)".to_owned())
            })?;
        self.symmetric_state.mix_key(&s.serialize()[..]);

        // If the initiator doesn't know our static key, then this operation
        // will fail.
        self.symmetric_state
            .decrypt_and_hash(&[], act_one.tag())
            .map_err(|err| {
                dbg!(&err);
                // TODO(mkl): why it is IO error, and not Crypto error
                HandshakeError::Io(err, "cannot decrypt_and_hash during receive ActOne".to_owned())
            })?;

        Ok(HandshakeInActOne {
            base: self,
            remote_ephemeral: remote_ephemeral,
        })
    }
}

pub struct HandshakeOut {
    contexts: (Secp256k1<SignOnly>, Secp256k1<VerifyOnly>),
    symmetric_state: SymmetricState,
    local_static: SecretKey,
    remote_static: PublicKey,
    pub ephemeral_gen: fn() -> SecretKey,
}

impl HandshakeOut {
    pub fn new(local_secret: SecretKey, remote_public: PublicKey) -> Result<Self, EcdsaError> {
        let mut symmetric_state = SymmetricState::new(PROTOCOL_NAME);
        symmetric_state.mix_hash("lightning".as_bytes());
        symmetric_state.mix_hash(&remote_public.serialize());

        Ok(HandshakeOut {
            contexts: (Secp256k1::signing_only(), Secp256k1::verification_only()),
            symmetric_state: symmetric_state,
            local_static: local_secret,
            remote_static: remote_public,
            // TODO(mkl): is it crypto secure to use this source of randomness
            ephemeral_gen: || {
                SecretKey::new(&mut rand::thread_rng())
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
    pub fn gen_act_one(mut self) -> Result<(ActOne, HandshakeOutActOne), HandshakeError> {
        use common_types::ac::SecretKey;

        let contexts = &self.contexts;

        // e
        let local_ephemeral = (self.ephemeral_gen)();

        let local_ephemeral_pub = PublicKey::from_secret_key(&contexts.0, &local_ephemeral);
        let ephemeral = local_ephemeral_pub.serialize();
        self.symmetric_state.mix_hash(&ephemeral);

        // es
        let s = local_ephemeral
            .dh(&contexts.1, &self.remote_static)
            .map_err(|err| {
                HandshakeError::Crypto(err, "Cannot calculate Diffie-Hellman (es) during generation ActOne".to_owned())
            })?;
        self.symmetric_state.mix_key(&s.serialize()[..]);

        let auth_payload = self
            .symmetric_state
            .encrypt_and_hash(&[], &mut Vec::new())
            .map_err(|err| {
                HandshakeError::Io(err, "cannot encrypt_and_hash authentication payload during generation ActOne".to_owned())
            })?;

        let act_one = ActOne::new(HandshakeVersion::_0, ephemeral, auth_payload);
        let handshake_act_one = HandshakeOutActOne {
            base: self,
            local_ephemeral: local_ephemeral,
        };
        Ok((act_one, handshake_act_one))
    }

    #[cfg(test)]
    pub fn handshake_digest(&self) -> [u8; 32] {
        self.symmetric_state.handshake_digest()
    }
}

pub struct HandshakeInActOne {
    base: HandshakeIn,
    remote_ephemeral: PublicKey,
}

impl HandshakeInActOne {
    // gen_act_two generates the second packet (act two) to be sent from the
    // responder to the initiator. The packet for act two is identify to that of
    // act one, but then results in a different ECDH operation between the
    // initiator's and responder's ephemeral keys.
    //
    //    <- e, ee
    pub fn gen_act_two(mut self) -> Result<(ActTwo, HandshakeInActTwo), HandshakeError> {
        use common_types::ac::SecretKey;

        let contexts = &self.base.contexts;

        // e
        let local_ephemeral = (self.base.ephemeral_gen)();

        let local_ephemeral_pub = PublicKey::from_secret_key(&contexts.0, &local_ephemeral);
        let ephemeral = local_ephemeral_pub.serialize();
        self.base.symmetric_state.mix_hash(&ephemeral);

        // ee
        let s = local_ephemeral
            .dh(&contexts.1, &self.remote_ephemeral)
            .map_err(|err| {
                HandshakeError::Crypto(err, "Cannot compute Diffie-Hellman (ee) during generation ActTwo".to_owned())
            })?;
        self.base.symmetric_state.mix_key(&s.serialize()[..]);

        let auth_payload = self
            .base
            .symmetric_state
            .encrypt_and_hash(&[], &mut Vec::new())
            .map_err(|err | {
                HandshakeError::Io(err, "cannot encrypt_and_hash authentication payload in generation ActTwo".to_owned())
            })?;

        let act_two = ActTwo(ActOne::new(HandshakeVersion::_0, ephemeral, auth_payload));
        let handshake = HandshakeInActTwo {
            base: self.base,
            local_ephemeral: local_ephemeral,
        };
        Ok((act_two, handshake))
    }
}

pub struct HandshakeOutActOne {
    base: HandshakeOut,
    local_ephemeral: SecretKey,
}

impl HandshakeOutActOne {
    // receive_act_two processes the second packet (act two) sent from the responder to
    // the initiator. A successful processing of this packet authenticates the
    // initiator to the responder.
    pub fn receive_act_two(mut self, act_two: ActTwo) -> Result<HandshakeOutActTwo, HandshakeError> {
        use common_types::ac::SecretKey;

        let contexts = &self.base.contexts;

        let ActTwo(inner) = act_two;

        // If the handshake version is unknown, then the handshake fails
        // immediately.
        if let Err(()) = inner.version() {
            let msg = format!("Act Two: invalid handshake version: {}", inner.bytes[0]);
            return Err(HandshakeError::UnknownHandshakeVersion(msg));
        }

        // e
        let remote_ephemeral = inner.key().map_err(|err|{
            HandshakeError::Crypto(err, "cannot obtain remote ephemeral public key from bytes in receive ActTwo".to_owned())
        })?;
        self.base
            .symmetric_state
            .mix_hash(&remote_ephemeral.serialize());

        // ee
        let s = self.local_ephemeral
            .dh(&contexts.1, &remote_ephemeral)
            .map_err(|err| {
                HandshakeError::Crypto(err, "cannot compute Diffie-Hellman public key (ee) in receive ActTwo".to_owned())
            })?;
        self.base.symmetric_state.mix_key(&s.serialize()[..]);

        self.base
            .symmetric_state
            .decrypt_and_hash(&mut Vec::new(), inner.tag())
            .map_err(|err| {
                dbg!(&err);
                // TODO(mkl): is it really an IO error ?
                HandshakeError::Io(err, "cannot decrypt_and_has during receive ActTwo".to_owned())
            })?;

        Ok(HandshakeOutActTwo {
            base: self.base,
            remote_ephemeral: remote_ephemeral,
        })
    }
}

pub struct HandshakeInActTwo {
    base: HandshakeIn,
    local_ephemeral: SecretKey,
}

impl HandshakeInActTwo {
    // receive_act_three processes the final act (act three) sent from the initiator to
    // the responder. After processing this act, the responder learns of the
    // initiator's static public key. Decryption of the static key serves to
    // authenticate the initiator to the responder.
    pub fn receive_act_three(mut self, act_three: ActThree) -> Result<Machine, HandshakeError> {
        use common_types::ac::SecretKey;

        let contexts = &self.base.contexts;

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
            .map_err(|err| {
                dbg!(&err);
                // TODO(mkl): is it really an IO error?
                HandshakeError::Io(err, "cannot decrypt_and_hash during receive ActThree".to_owned())
            })?;
        let remote_static = PublicKey::from_slice(&remote_pub)
            .map_err(|err|{
                dbg!(&err);
                HandshakeError::Crypto(err, "cannot create remote pubkey from bytes during receive ActThree".to_owned())
            })?;

        // se
        let se = self.local_ephemeral
            .dh(&contexts.1, &remote_static)
            .map_err(|err| {
                HandshakeError::Crypto(err, "Cannot compute Diffie-Helman public key (se) during receive ActThree".to_owned())
            })?;
        self.base.symmetric_state.mix_key(&se.serialize()[..]);

        self.base
            .symmetric_state
            .decrypt_and_hash(&[], act_three.tag_second())
            .map_err(|err| {
                dbg!(&err);
                // TODO(mkl): is it really an IO error?
                HandshakeError::Io(err, "cannot decrypt_and_hash during receive ActThree".to_owned())
            })?;

        // With the final ECDH operation complete, derive the session sending
        // and receiving keys.
        // swap them
        #[cfg(test)]
        let chaining_key = self.base.symmetric_state.chaining_key();
        let (receive, send) = self.base.symmetric_state.into_pair();
        Ok(Machine {
            send_cipher: send,
            receive_cipher: receive,
            remote_static: remote_static,
            #[cfg(test)]
            chaining_key: chaining_key,
            message_buffer: Arc::new(RwLock::new([0; std::u16::MAX as usize])),
        })
    }
}

pub struct HandshakeOutActTwo {
    base: HandshakeOut,
    remote_ephemeral: PublicKey,
}

impl HandshakeOutActTwo {
    // gen_act_three creates the final (act three) packet of the handshake. Act three
    // is to be sent from the initiator to the responder. The purpose of act three
    // is to transmit the initiator's public key under strong forward secrecy to
    // the responder. This act also includes the final ECDH operation which yields
    // the final session.
    //
    //    -> s, se
    pub fn gen_act_three(mut self) -> Result<(ActThree, Machine), HandshakeError> {
        use secp256k1::constants::PUBLIC_KEY_SIZE;
        use common_types::ac::SecretKey;

        let contexts = &self.base.contexts;

        let local_static_pub = PublicKey::from_secret_key(&contexts.0, &self.base.local_static);
        let our_pubkey = local_static_pub.serialize();
        let mut cipher_text = Vec::with_capacity(PUBLIC_KEY_SIZE);
        let tag = self
            .base
            .symmetric_state
            .encrypt_and_hash(&our_pubkey, &mut cipher_text)
            // TODO(mkl): is it really an IO error
            .map_err(|err| {
                HandshakeError::Io(err, "cannot encrypt_and_hash during generate ActThree".to_owned())
            })?;

        let s = self.base.local_static
            .dh(&contexts.1, &self.remote_ephemeral)
            .map_err(|err| {
                HandshakeError::Crypto(err, "cannot calculate Diffie-Hellman during generate ActThree".to_owned())
            })?;
        self.base.symmetric_state.mix_key(&s.serialize()[..]);

        let auth_payload = self
            .base
            .symmetric_state
            .encrypt_and_hash(&[], &mut Vec::new())
            // TODO(mkl): is it really an IO error
            .map_err(|err| {
                HandshakeError::Io(err, "cannot encrypt_and_hash authentication payload during generate ActThree".to_owned())
            })?;

        let act_three = ActThree::new(HandshakeVersion::_0, cipher_text, tag, auth_payload);

        // With the final ECDH operation complete, derive the session sending
        // and receiving keys.

        #[cfg(test)]
        let chaining_key = self.base.symmetric_state.chaining_key();
        let (send, receive) = self.base.symmetric_state.into_pair();
        let machine = Machine {
            send_cipher: send,
            receive_cipher: receive,
            remote_static: self.base.remote_static,
            #[cfg(test)]
            chaining_key: chaining_key,
            message_buffer: Arc::new(RwLock::new([0; std::u16::MAX as usize])),
        };

        Ok((act_three, machine))
    }
}

use bytes::BytesMut;
use binformat::{BinarySD, WireError};
use serde::{Serialize, de::DeserializeOwned};
use std::io::Write;

pub struct Machine {
    send_cipher: CipherState,
    receive_cipher: CipherState,
    remote_static: PublicKey,
    #[cfg(test)]
    chaining_key: [u8; 32],
    message_buffer: Arc<RwLock<[u8; std::u16::MAX as usize]>>,
}

impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Machine")
            .field("send_cipher", &self.send_cipher)
            .field("receive_cipher", &self.receive_cipher)
            .field("remote_static", &self.remote_static)
            .finish()
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
    pub fn remote_static(&self) -> PublicKey {
        self.remote_static.clone()
    }

    pub fn write<T>(&mut self, item: T, extra_data: Vec<u8>, dst: &mut BytesMut) -> Result<(), WireError>
    where
        T: Serialize,
    {
        use bytes::BufMut;

        let length = {
            let mut buffer = self.message_buffer.write().unwrap();
            let mut cursor = io::Cursor::new(buffer.as_mut());
            BinarySD::serialize(&mut cursor, &item)?;
            cursor.write(extra_data.as_slice()).map_err(WireError::from)?;
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
            &self.message_buffer.read().unwrap()[..length],
        )?;
        dst.put_slice(&tag[..]);

        Ok(())
    }


    pub fn read<T>(&mut self, src: &mut BytesMut) -> Result<Option<(T, Vec<u8>)>, WireError>
        where
            T: DeserializeOwned + fmt::Debug,
    {
        // TODO(mkl): make logging messages optional
//        dbg!(self.read_int(src))
        let r = self.read_int(src);
        eprintln!("{:#?}", &r);
        r
    }

    pub fn read_int<T>(&mut self, src: &mut BytesMut) -> Result<Option<(T, Vec<u8>)>, WireError>
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
                self.receive_cipher
                    .decrypt(&[], &mut plain.as_mut(), cipher.as_ref(), tag)
                    .map_err(|e| match e {
                        DecryptError::IoError(e) => dbg!(e.into()),
                        DecryptError::TagMismatch => {
                            println!("TagMismatch error 1");
                            WireError::custom("tag")
                        },
                    })?;

                let length: u16 = BinarySD::deserialize(&plain[..])?;
                length as usize
            };

            if src.len() < length + MAC_SIZE {
                Ok(None)
            } else {
                let cipher = src.split_to(length);
                let tag = tag(src);

                self.receive_cipher
                    .decrypt(
                        &[],
                        &mut self.message_buffer.write().unwrap().as_mut(),
                        cipher.as_ref(),
                        tag,
                    ).map_err(|e| match e {
                        DecryptError::IoError(e) => dbg!(e.into()),
                        DecryptError::TagMismatch => {
                            println!("TagMismatch error 2");
                            WireError::custom("tag")
                        },
                    })?;

                let buffer = self.message_buffer.read().unwrap();
                let mut cursor = io::Cursor::new(buffer.as_ref());
                BinarySD::deserialize(&mut cursor)
                    .map(|m| {
                        let read = cursor.position() as usize;
                        let extra_size = length - read;
                        let mut extra_data = Vec::with_capacity(extra_size);
                        extra_data.resize(extra_size, 0);
                        extra_data.as_mut_slice().copy_from_slice(&cursor.into_inner()[read..length]);
                        (m, extra_data)
                    })
                    .map(Some)
            }
        }
    }

    #[cfg(test)]
    pub fn send_cipher_key(&self) -> [u8; 32] {
        self.send_cipher.secret_key()
    }

    #[cfg(test)]
    pub fn receive_cipher_key(&self) -> [u8; 32] {
        self.receive_cipher.secret_key()
    }

    #[cfg(test)]
    pub fn chaining_key(&self) -> [u8; 32] {
        self.chaining_key.clone()
    }
}
