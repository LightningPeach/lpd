pub trait PublicKey
where
    Self: Sized + Eq,
{}

pub trait SecretKey
where
    Self: Sized + Eq,
{
    type PublicKey: PublicKey;

    type Context;

    fn from_raw<T>(v: T) -> Self
    where
        T: AsRef<[u8]>;

    fn paired(&self, context: &Self::Context) -> Self::PublicKey;
}

pub trait Data<H> {
    type ContentToHash;

    fn as_ref_content(&self) -> &Self::ContentToHash;
    fn double_hash(&self) -> H;
}

pub trait SignError {
    fn invalid_signature(&self) -> bool;
}

/// Resource Acquisition Is Initialization and... Validation
pub trait Signed
where
    Self: Sized,
{
    type SecretKey: SecretKey;

    type Error: SignError;

    type Hash;

    type Data: Data<Self::Hash>;

    type SigningContext;

    type VerificationContext;

    fn sign(data: Self::Data, context: &Self::SigningContext, secret_key: &Self::SecretKey) -> Self;

    fn check(&self, context: &Self::VerificationContext, public_key: &<Self::SecretKey as SecretKey>::PublicKey) -> Result<(), Self::Error>;

    fn verify(self, context: &Self::VerificationContext, public_key: &<Self::SecretKey as SecretKey>::PublicKey) -> Result<Self::Data, Self::Error>;

    fn verify_key_inside<F>(self, context: &Self::VerificationContext, get_public_key: F) -> Result<Self::Data, Self::Error>
    where
        F: FnOnce(&<Self::Data as Data<Self::Hash>>::ContentToHash) -> &<Self::SecretKey as SecretKey>::PublicKey;
}
