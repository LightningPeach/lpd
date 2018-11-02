use super::CipherState;
use std::{fmt, io};

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

    // handshake_digest is the cumulative hash digest of all handshake
    // messages sent from start to finish. This value is never transmitted
    // to the other side, but will be used as the AD when
    // encrypting/decrypting messages using our AEAD construction.
    handshake_digest: [u8; 32],
}

impl fmt::Debug for SymmetricState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#"
        cipher_state:     {:?}
        chaining_key:     {:?}
        handshake_digest: {:?}
        "#,
            self.cipher_state,
            hex::encode(self.chaining_key),
            hex::encode(self.handshake_digest),
        )
    }
}

impl SymmetricState {
    pub fn new(protocol_name: &str) -> Self {
        use sha2::{Digest, Sha256};

        let digest = {
            let mut hasher = Sha256::default();
            hasher.input(protocol_name.as_bytes());
            let mut digest: [u8; 32] = [0; 32];
            digest.copy_from_slice(&hasher.result()[..]);
            digest
        };

        SymmetricState {
            cipher_state: CipherState::new([0; 32], [0; 32]),
            chaining_key: digest,
            handshake_digest: digest,
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
        let okm = hkdf.expand(&[], 64);

        self.chaining_key.copy_from_slice(&okm.as_slice()[..32]);

        let mut temp_key = [0; 32];
        temp_key.copy_from_slice(&okm.as_slice()[32..]);

        self.cipher_state = CipherState::new([0; 32], temp_key);
    }

    // mix_hash hashes the passed input data into the cumulative handshake digest.
    // The running result of this value (h) is used as the associated data in all
    // decryption/encryption operations.
    pub fn mix_hash(&mut self, data: &[u8]) {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::default();
        hasher.input(&self.handshake_digest);
        hasher.input(data);

        self.handshake_digest.copy_from_slice(&hasher.result()[..]);
    }

    // encrypt_and_hash returns the authenticated encryption of the passed plaintext.
    // When encrypting the handshake digest (h) is used as the associated data to
    // the AEAD cipher
    pub fn encrypt_and_hash(
        &mut self,
        plain_text: &[u8],
        cipher_text: &mut Vec<u8>,
    ) -> Result<[u8; MAC_SIZE], io::Error> {
        let tag = self
            .cipher_state
            .encrypt(&self.handshake_digest, cipher_text, plain_text)?;

        // To be compliant with golang's implementation of chacha20poly1305 and brontide packages
        // we concatenate cipher_text and mac for mixing with internal state.
        let mut cipher_text_with_mac = cipher_text.clone();
        cipher_text_with_mac.extend(&tag);

        self.mix_hash(&mut cipher_text_with_mac);

        Ok(tag)
    }

    // decrypt_and_hash returns the authenticated decryption of the passed
    // ciphertext.  When encrypting the handshake digest (h) is used as the
    // associated data to the AEAD cipher.
    pub fn decrypt_and_hash(
        &mut self,
        cipher_text: &[u8],
        tag: [u8; MAC_SIZE],
    ) -> Result<Vec<u8>, io::Error> {
        let mut plain_text = Vec::new();
        self.cipher_state
            .decrypt(&self.handshake_digest, &mut plain_text, cipher_text, tag)?;

        let mut cipher_text_with_mac = cipher_text.to_vec();
        cipher_text_with_mac.extend(&tag);

        self.mix_hash(&cipher_text_with_mac);

        Ok(plain_text)
    }

    #[cfg(test)]
    pub fn inspect(&self, digest: &str) {
        assert_eq!(hex::encode(&self.handshake_digest[..]), digest);
    }
}
