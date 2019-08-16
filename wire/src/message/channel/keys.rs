use dependencies::secp256k1;

use secp256k1::{SecretKey, PublicKey};
use serde_derive::{Serialize, Deserialize};
use common_types::ac;
use super::super::types::RawPublicKey;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ChannelKeys {
    pub funding: RawPublicKey,
    pub revocation: RawPublicKey,
    pub payment: RawPublicKey,
    pub delayed_payment: RawPublicKey,
    pub htlc: RawPublicKey,
    pub first_per_commitment: RawPublicKey,
}

impl ChannelKeys {
    pub fn new(private: &ChannelPrivateKeys) -> Self {
        use secp256k1::Secp256k1;
        let context = Secp256k1::signing_only();
        ChannelKeys {
            funding: ac::SecretKey::paired(&private.funding, &context).into(),
            revocation: ac::SecretKey::paired(&private.revocation, &context).into(),
            payment: ac::SecretKey::paired(&private.payment, &context).into(),
            delayed_payment: ac::SecretKey::paired(&private.delayed_payment, &context).into(),
            htlc: ac::SecretKey::paired(&private.htlc, &context).into(),
            first_per_commitment: ac::SecretKey::paired(&private.first_per_commitment, &context).into(),
        }
    }

    pub fn funding(&self) -> &PublicKey {
        &self.funding.as_ref()
    }

    pub fn revocation(&self) -> &PublicKey {
        &self.revocation.as_ref()
    }

    pub fn payment(&self) -> &PublicKey {
        &self.payment.as_ref()
    }

    pub fn delayed_payment(&self) -> &PublicKey {
        &self.delayed_payment.as_ref()
    }

    pub fn htlc(&self) -> &PublicKey {
        &self.htlc.as_ref()
    }

    pub fn first_per_commitment(&self) -> &PublicKey {
        &self.first_per_commitment.as_ref()
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ChannelPrivateKeys {
    funding: SecretKey,
    revocation: SecretKey,
    payment: SecretKey,
    delayed_payment: SecretKey,
    htlc: SecretKey,
    first_per_commitment: SecretKey,
}

impl ChannelPrivateKeys {
    pub fn funding_sk(&self) -> &SecretKey {
        &self.funding
    }

    pub fn revocation_sk(&self) -> &SecretKey {
        &self.revocation
    }

    pub fn payment_sk(&self) -> &SecretKey {
        &self.payment
    }

    pub fn delayed_payment_sk(&self) -> &SecretKey {
        &self.delayed_payment
    }

    pub fn htlc_sk(&self) -> &SecretKey {
        &self.htlc
    }

    pub fn first_per_commitment_sk(&self) -> &SecretKey {
        &self.first_per_commitment
    }
}

mod rand_m {
    use dependencies::secp256k1;
    use dependencies::rand;

    use secp256k1::SecretKey;
    use super::ChannelPrivateKeys;
    use rand::distributions::{Distribution, Standard};
    use rand::Rng;

    fn rand_secret_key<R: Rng + ?Sized>(rng: &mut R) -> SecretKey {
        let random_bytes = rng.gen::<[u8; 32]>();
        SecretKey::from_slice(&random_bytes).unwrap()
    }

    impl Distribution<ChannelPrivateKeys> for Standard {
        // TODO(mkl): fix this to use distribution
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ChannelPrivateKeys {
            ChannelPrivateKeys {
                funding: rand_secret_key(rng),
                revocation: rand_secret_key(rng),
                payment: rand_secret_key(rng),
                delayed_payment: rand_secret_key(rng),
                htlc: rand_secret_key(rng),
                first_per_commitment: rand_secret_key(rng),
            }
        }
    }
}
