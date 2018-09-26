use serde::Serialize;
use ::WireError;

use secp256k1::Error as Secp256k1Error;
use secp256k1::Message as Secp256k1Message;

use super::types::SecretKey;
use super::types::PublicKey;
use super::types::Signature;

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Signed<T> where T: DataToSign {
    pub signature: Signature,
    pub value: T,
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct SignedData<T>(pub T) where T: Serialize;

pub trait DataToSign: Serialize {
    type Inner: Serialize;

    fn as_ref_data(&self) -> &Self::Inner;

    fn hash(&self) -> Result<Secp256k1Message, SignError> {
        use self::SignError::*;
        use sha2::Sha256;
        use digest::FixedOutput;
        use digest::Input;
        use ::BinarySD;

        let mut v = Vec::new();
        let data = self.as_ref_data();
        BinarySD::serialize(&mut v, data).map_err(WireError)?;
        let mut first = Sha256::default();
        first.process(v.as_slice());
        let mut second = Sha256::default();
        second.process(first.fixed_result().as_slice());

        Secp256k1Message::from_slice(second.fixed_result().as_slice())
            .map_err(Secp256k1Error)
    }
}

// recursion base
impl<T> DataToSign for SignedData<T> where T: Serialize {
    type Inner = T;

    fn as_ref_data(&self) -> &Self::Inner {
        &self.0
    }
}

// recursion step
impl<T> DataToSign for Signed<T> where T: DataToSign {
    type Inner = T::Inner;

    fn as_ref_data(&self) -> &Self::Inner {
        &self.value.as_ref_data()
    }
}

#[derive(Debug)]
pub enum SignError {
    WireError(WireError),
    Secp256k1Error(Secp256k1Error),
    IncorrectSignature,
}

impl<T> Signed<T> where T: DataToSign {
    pub fn sign(value: T, key: &SecretKey) -> Result<Self, SignError> {
        use self::SignError::*;
        use secp256k1::Secp256k1;

        let msg = value.hash()?;
        Secp256k1::new().sign(&msg, key.as_ref())
            .map_err(Secp256k1Error)
            .map(|s| {
                Signed {
                    signature: Signature::from(s),
                    value: value,
                }
            })
    }

    fn check(&self, public_key: &PublicKey) -> Result<(), SignError> {
        use secp256k1::Secp256k1;

        let msg = self.hash()?;
        Secp256k1::new().verify(&msg, self.signature.as_ref(), public_key.as_ref())
            .map_err(|e| match e {
                Secp256k1Error::IncorrectSignature => SignError::IncorrectSignature,
                e @ _ => SignError::Secp256k1Error(e),
            })
    }

    pub fn verify(self, public_key: &PublicKey) -> Result<T, SignError> {
        self.check(public_key)?;
        Ok(self.value)
    }

    pub fn verify_any_of_two(self, first: &PublicKey, second: &PublicKey) -> Result<(), SignError> {
        self.check(first).or_else(|e| match e {
            SignError::IncorrectSignature => self.check(second),
            e @ _ => Err(e),
        })
    }

    // verify using the public key owned by the inner content
    // the closure should borrow from inner
    pub fn verify_owned<F>(self, borrow_from_inner: F) -> Result<T, SignError>
    where
        F: Fn(&<Self as DataToSign>::Inner) -> &PublicKey,
    {
        self.check(borrow_from_inner(self.as_ref_data()))?;
        Ok(self.value)
    }
}
