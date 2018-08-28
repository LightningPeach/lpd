
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Hash256 {
    data: [u8; 32],
}

impl Hash256 {
    pub const BITCOIN_CHAIN_HASH: Self = Hash256 {
        data: hex!("6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000"),
    };
}

mod sha2 {
    use sha2::Sha256;
    use super::Hash256;

    impl From<Sha256> for Hash256 {
        fn from(v: Sha256) -> Self {
            let _ = v;
            unimplemented!()
        }
    }

    impl From<Hash256> for Sha256 {
        fn from(v: Hash256) -> Self {
            let _ = v;
            unimplemented!()
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
