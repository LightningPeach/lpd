use super::PublicKey;
use super::SecretKey;

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
        ChannelKeys {
            funding: PublicKey::paired(&private.funding),
            revocation: PublicKey::paired(&private.revocation),
            payment: PublicKey::paired(&private.payment),
            delayed_payment: PublicKey::paired(&private.delayed_payment),
            htlc: PublicKey::paired(&private.htlc),
            first_per_commitment: PublicKey::paired(&private.first_per_commitment),
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
mod rand {
    use super::ChannelPrivateKeys;
    use rand::distributions::Distribution;
    use rand::distributions::Standard;
    use rand::Rng;

    impl Distribution<ChannelPrivateKeys> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ChannelPrivateKeys {
            ChannelPrivateKeys {
                funding: self.sample(rng),
                revocation: self.sample(rng),
                payment: self.sample(rng),
                delayed_payment: self.sample(rng),
                htlc: self.sample(rng),
                first_per_commitment: self.sample(rng),
            }
        }
    }

}
