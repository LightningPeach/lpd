pub trait PublicKey
where
    Self: Sized + Eq,
{}

pub trait SecretKey<H>
where
    Self: Sized + Eq,
{
    type Error;

    type PublicKey: PublicKey;

    type SigningContext;

    type VerificationContext;

    fn from_raw<T>(v: T) -> Self
    where
        T: AsRef<[u8]>;

    fn paired(&self, context: &Self::SigningContext) -> Self::PublicKey;

    fn dh(&self, context: &Self::VerificationContext, pk: &Self::PublicKey) -> Result<H, Self::Error>;
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
pub trait Signed<H>
where
    Self: Sized,
{
    type Error: SignError;

    type SecretKey: SecretKey<H>;

    type Data: Data<H>;

    fn sign(data: Self::Data, context: &<Self::SecretKey as SecretKey<H>>::SigningContext, secret_key: &Self::SecretKey) -> Self;

    fn check(&self, context: &<Self::SecretKey as SecretKey<H>>::VerificationContext, public_key: &<Self::SecretKey as SecretKey<H>>::PublicKey) -> Result<(), Self::Error>;

    fn verify(self, context: &<Self::SecretKey as SecretKey<H>>::VerificationContext, public_key: &<Self::SecretKey as SecretKey<H>>::PublicKey) -> Result<Self::Data, Self::Error>;

    fn verify_key_inside<F>(self, context: &<Self::SecretKey as SecretKey<H>>::VerificationContext, get_public_key: F) -> Result<Self::Data, Self::Error>
    where
        F: FnOnce(&<Self::Data as Data<H>>::ContentToHash) -> &<Self::SecretKey as SecretKey<H>>::PublicKey;
}
