#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Hash {
    data: [u8; 32],
}

#[cfg(test)]
mod rand {
    use super::Hash;

    use rand::distributions::Distribution;
    use rand::distributions::Standard;
    use rand::Rng;

    impl Distribution<Hash> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Hash {
            let mut rng = rng;
            let rnd_bytes: Vec<u8> = self.sample_iter(&mut rng).take(32).collect();
            let mut this = Hash { data: [0u8; 32], };
            this.data.copy_from_slice(rnd_bytes.as_slice());
            this
        }
    }
}
