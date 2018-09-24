
#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub struct Hash256 {
    data: [u8; 32],
}

impl From<Hash256> for [u8; 32] {
    fn from(h: Hash256) -> Self {
        return h.data;
    }
}

impl Hash256 {
    pub const BITCOIN_CHAIN_HASH: Self = Hash256 {
        data: hex!("6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000"),
    };

    pub const TEST_HASH: Self = Hash256 {
        data: hex!("38faad210ccb4b018c866049827661643433f1a261a54a8b3faa9e682341158d"),
    };
}

mod debug {
    use super::Hash256;

    use hex::encode;
    use std::fmt;

    impl fmt::Debug for Hash256 {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Hash256 [ {} ]", encode(&self.data[0..]))
        }
    }
}

mod secp256k1 {
    use super::Hash256;
    use secp256k1::Message;
    use secp256k1::constants;

    impl From<Hash256> for Message {
        fn from(v: Hash256) -> Self {
            <Message as From<[u8; constants::MESSAGE_SIZE]>>::from(v.data)
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod rand {
    use super::Hash256;

    use rand::distributions::Distribution;
    use rand::distributions::Standard;
    use rand::Rng;

    impl Distribution<Hash256> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Hash256 {
            let mut rng = rng;
            let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(32).collect();
            let mut this = Hash256 { data: [0u8; 32], };
            this.data.copy_from_slice(rnd_bytes.as_slice());
            this
        }
    }
}
