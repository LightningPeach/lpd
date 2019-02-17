use secp256k1::{SecretKey, PublicKey};
use serde_derive::{Serialize, Deserialize};
use common_types::ac;
use super::super::types::RawPublicKey;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ChannelKeys {
    funding: RawPublicKey,
    revocation: RawPublicKey,
    payment: RawPublicKey,
    delayed_payment: RawPublicKey,
    htlc: RawPublicKey,
    first_per_commitment: RawPublicKey,
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
