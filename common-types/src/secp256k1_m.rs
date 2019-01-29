use secp256k1::{PublicKey, SecretKey, Message, Signature, Error, Secp256k1, SignOnly, VerifyOnly};
use super::ac;
use serde::Serialize;
use serde_derive::{Serialize, Deserialize};

impl ac::PublicKey for PublicKey {}

impl ac::SecretKey<Message> for SecretKey {
    type Error = Error;

    type PublicKey = PublicKey;

    type SigningContext = Secp256k1<SignOnly>;

    type VerificationContext = Secp256k1<VerifyOnly>;

    fn from_raw<T>(v: T) -> Self where T: AsRef<[u8]> {
        SecretKey::from_slice(v.as_ref()).unwrap()
    }

    fn paired(&self, context: &Self::SigningContext) -> Self::PublicKey {
        PublicKey::from_secret_key(&context, self)
    }

    fn dh(&self, context: &Self::VerificationContext, pk: &Self::PublicKey) -> Result<Message, Self::Error> {
        use sha2::{Sha256, Digest};

        let mut pk_cloned = pk.clone();
        pk_cloned.mul_assign(&context, &self[..])?;

        let mut hasher = Sha256::default();
        hasher.input(&pk_cloned.serialize()[..]);
        let hash = hasher.result();

        Message::from_slice(hash.as_slice())
    }
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Data<T>(pub T)
where
    T: Serialize;

impl<T> ac::Data<Message> for Data<T>
where
    T: Serialize,
{
    type ContentToHash = T;

    fn as_ref_content(&self) -> &Self::ContentToHash {
        &self.0
    }

    fn double_hash(&self) -> Message {
        use binformat::BinarySD;

        fn hash256<T>(v: T) -> impl AsRef<[u8]>
        where
            T: AsRef<[u8]>,
        {
            use sha2::Sha256;
            use digest::FixedOutput;
            use digest::Input;

            let mut hasher = Sha256::default();
            hasher.input(v.as_ref());
            hasher.fixed_result()
        }

        let mut v = Vec::new();
        BinarySD::serialize(&mut v, &self.0).unwrap();
        let h = hash256(hash256(v));
        Message::from_slice(h.as_ref()).unwrap()
    }
}

impl ac::SignError for Error {
    fn invalid_signature(&self) -> bool {
        match self {
            &Error::IncorrectSignature => true,
            _ => false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Signed<T>
where
    T: ac::Data<Message>,
{
    pub signature: Signature,
    data: T,
}

impl<T> ac::Data<Message> for Signed<T>
where
    T: ac::Data<Message>,
{
    type ContentToHash = T::ContentToHash;

    fn as_ref_content(&self) -> &Self::ContentToHash {
        self.data.as_ref_content()
    }

    fn double_hash(&self) -> Message {
        self.data.double_hash()
    }
}

impl<T> ac::Signed<Message> for Signed<T>
where
    T: ac::Data<Message>,
{
    type Error = Error;

    type SecretKey = SecretKey;

    type Data = T;

    fn sign(data: Self::Data, context: &<Self::SecretKey as ac::SecretKey<Message>>::SigningContext, secret_key: &Self::SecretKey) -> Self {
        let message = data.double_hash();
        let signature = context.sign(&message, secret_key);
        Signed {
            signature: signature,
            data: data,
        }
    }

    fn check(&self, context: &<Self::SecretKey as ac::SecretKey<Message>>::VerificationContext, public_key: &<Self::SecretKey as ac::SecretKey<Message>>::PublicKey) -> Result<(), Self::Error> {
        let message = self.data.double_hash();
        context.verify(&message, &self.signature, public_key)
    }

    fn verify(self, context: &<Self::SecretKey as ac::SecretKey<Message>>::VerificationContext, public_key: &<Self::SecretKey as ac::SecretKey<Message>>::PublicKey) -> Result<Self::Data, Self::Error> {
        self.check(context, public_key).map(|()| self.data)
    }

    fn verify_key_inside<F>(self, context: &<Self::SecretKey as ac::SecretKey<Message>>::VerificationContext, get_public_key: F) -> Result<Self::Data, Self::Error>
    where
        F: FnOnce(&<Self::Data as ac::Data<Message>>::ContentToHash) -> &<Self::SecretKey as ac::SecretKey<Message>>::PublicKey,
    {
        let public_key = get_public_key(&self.data.as_ref_content()).clone();
        self.verify(context, &public_key)
    }
}
