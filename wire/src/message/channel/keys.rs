use secp256k1::{SecretKey, PublicKey};
use serde_derive::{Serialize, Deserialize};
use common_types::ac;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ChannelKeys {
    funding: PublicKey,
    revocation: PublicKey,
    payment: PublicKey,
    delayed_payment: PublicKey,
    htlc: PublicKey,
    first_per_commitment: PublicKey,
}

impl ChannelKeys {
    pub fn new(private: &ChannelPrivateKeys) -> Self {
        use secp256k1::Secp256k1;
        let context = Secp256k1::signing_only();
        ChannelKeys {
            funding: ac::SecretKey::paired(&private.funding, &context),
            revocation: ac::SecretKey::paired(&private.revocation, &context),
            payment: ac::SecretKey::paired(&private.payment, &context),
            delayed_payment: ac::SecretKey::paired(&private.delayed_payment, &context),
            htlc: ac::SecretKey::paired(&private.htlc, &context),
            first_per_commitment: ac::SecretKey::paired(&private.first_per_commitment, &context),
        }
    }

    pub fn funding(&self) -> &PublicKey {
        &self.funding
    }

    pub fn revocation(&self) -> &PublicKey {
        &self.revocation
    }

    pub fn payment(&self) -> &PublicKey {
        &self.payment
    }

    pub fn delayed_payment(&self) -> &PublicKey {
        &self.delayed_payment
    }

    pub fn htlc(&self) -> &PublicKey {
        &self.htlc
    }

    pub fn first_per_commitment(&self) -> &PublicKey {
        &self.first_per_commitment
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

#[cfg(any(test, feature = "testing"))]
mod rand_m {
    use super::ChannelPrivateKeys;
    use rand::Rand;
    use rand::Rng;

    impl Rand for ChannelPrivateKeys {
        fn rand<R: Rng>(rng: &mut R) -> Self {
            use secp256k1::SecretKey;

            ChannelPrivateKeys {
                funding: SecretKey::new(rng),
                revocation: SecretKey::new(rng),
                payment: SecretKey::new(rng),
                delayed_payment: SecretKey::new(rng),
                htlc: SecretKey::new(rng),
                first_per_commitment: SecretKey::new(rng),
            }
        }
    }

}
