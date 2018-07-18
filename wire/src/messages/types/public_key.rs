#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct PublicKey {
    data: [u8; 32],
    last: u8,
}

#[cfg(test)]
mod rand {
    use super::PublicKey;

    use rand::distributions::Distribution;
    use rand::distributions::Standard;
    use rand::Rng;

    impl Distribution<PublicKey> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PublicKey {
            let mut rng = rng;
            let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(32).collect();
            let mut this = PublicKey { data: [0u8; 32], last: rng.gen() };
            this.data.copy_from_slice(rnd_bytes.as_slice());
            this
        }
    }
}
