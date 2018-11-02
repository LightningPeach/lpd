use std::{fmt, io};
use super::CipherState;

// TODO: needed MAC type encapsulating the array [u8; MAC_SIZE]
const MAC_SIZE: usize = 16;

// SymmetricState encapsulates a cipherState object and houses the ephemeral
// handshake digest state. This struct is used during the handshake to derive
// new shared secrets based off of the result of ECDH operations. Ultimately,
// the final key yielded by this struct is the result of an incremental
// Triple-DH operation.
pub struct SymmetricState {
    cipher_state: CipherState,

    // chaining_key is used as the salt to the HKDF function to derive a new
    // chaining key as well as a new tempKey which is used for
    // encryption/decryption.
    pub chaining_key: [u8; 32],

    // temp_key is the latter 32 bytes resulted from the latest HKDF
    // iteration. This key is used to encrypt/decrypt any handshake
    // messages or payloads sent until the next DH operation is executed.
    temp_key: [u8; 32],

    // handshake_digest is the cumulative hash digest of all handshake
    // messages sent from start to finish. This value is never transmitted
    // to the other side, but will be used as the AD when
    // encrypting/decrypting messages using our AEAD construction.
    handshake_digest: [u8; 32],
}

impl fmt::Debug for SymmetricState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"
        cipher_state:     {:?}
        chaining_key:     {:?}
        temp_key:         {:?}
        handshake_digest: {:?}
        "#, self.cipher_state, hex::encode(self.chaining_key),
               hex::encode(self.temp_key), hex::encode(self.handshake_digest),
        )
    }
}

// TODO: add new constructors
impl SymmetricState {
    pub fn new() -> Self {
        Self {
            cipher_state: CipherState::new(),
            chaining_key: [0; 32],
            temp_key: [0; 32],
            handshake_digest: [0; 32],
        }
    }

    // mix_key is implements a basic HKDF-based key ratchet. This method is called
    // with the result of each DH output generated during the handshake process.
    // The first 32 bytes extract from the HKDF reader is the next chaining key,
    // then latter 32 bytes become the temp secret key using within any future AEAD
    // operations until another DH operation is performed.
    pub fn mix_key(&mut self, input: &[u8]) {
        use sha2::Sha256;

        let mut salt: [u8; 32] = [0; 32];
        salt.copy_from_slice(&self.chaining_key);
        let hkdf = hkdf::Hkdf::<Sha256>::extract(Some(&salt), input);

        let info: &[u8] = &[];
        let okm = hkdf.expand(info, 64);

        self.chaining_key.copy_from_slice(&okm.as_slice()[..32]);
        self.temp_key.copy_from_slice(&okm.as_slice()[32..]);

        self.cipher_state.initialize_key(self.temp_key);
    }

    // mix_hash hashes the passed input data into the cumulative handshake digest.
    // The running result of this value (h) is used as the associated data in all
    // decryption/encryption operations.
    pub fn mix_hash(&mut self, data: &[u8]) {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::default();
        hasher.input(&self.handshake_digest);
        hasher.input(data);

        self.handshake_digest.copy_from_slice(&hasher.result()[..]);
    }

    // encrypt_and_hash returns the authenticated encryption of the passed plaintext.
    // When encrypting the handshake digest (h) is used as the associated data to
    // the AEAD cipher
    pub fn encrypt_and_hash(&mut self, plaintext: &[u8], cipher_text: &mut Vec<u8>) -> Result<[u8; MAC_SIZE], io::Error> {
        let tag = self.cipher_state.encrypt(
            &self.handshake_digest, cipher_text, plaintext)?;

        // To be compliant with golang's implementation of chacha20poly1305 and brontide packages
        // we concatenate cipher_text and mac for mixing with internal state.
        let mut cipher_text_with_mac: Vec<u8> = Vec::new();
        for item in cipher_text.clone() {
            cipher_text_with_mac.push(item.clone());
        }
        for item in &tag {
            cipher_text_with_mac.push(item.clone());
        }

        self.mix_hash(&mut cipher_text_with_mac);

        Ok(tag)
    }

    // decrypt_and_hash returns the authenticated decryption of the passed
    // ciphertext.  When encrypting the handshake digest (h) is used as the
    // associated data to the AEAD cipher.
    pub fn decrypt_and_hash(&mut self, ciphertext: &[u8], tag: [u8; MAC_SIZE]) -> Result<Vec<u8>, io::Error> {
        let mut plaintext: Vec<u8> = Vec::new();
        self.cipher_state.decrypt(&self.handshake_digest, &mut plaintext, ciphertext, tag)?;

        let mut cipher_text_with_mac: Vec<u8> = Vec::new();
        for item in ciphertext.clone() {
            cipher_text_with_mac.push(item.clone());
        }
        for item in &tag {
            cipher_text_with_mac.push(item.clone());
        }

        self.mix_hash(&cipher_text_with_mac);

        Ok(plaintext)
    }

    // initialize_symmetric initializes the symmetric state by setting the handshake
    // digest (h) and the chaining key (ck) to protocol name.
    pub fn initialize_symmetric(&mut self, protocol_name: &[u8]) {
        use sha2::{Sha256, Digest};

        let empty: [u8; 32] = [0; 32];

        let mut hasher = Sha256::default();
        hasher.input(protocol_name);
        self.handshake_digest.copy_from_slice(&hasher.result()[..]);
        self.chaining_key = self.handshake_digest;
        self.cipher_state.initialize_key(empty);
    }
}
