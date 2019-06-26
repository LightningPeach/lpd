#[cfg(not(target_arch = "wasm32"))]
pub use self::regular::*;

#[cfg(not(target_arch = "wasm32"))]
mod regular {
    pub use secp256k1_c::*;
}

#[cfg(target_arch = "wasm32")]
pub use self::pure_rust::*;

#[cfg(target_arch = "wasm32")]
mod pure_rust {
    use std::marker::PhantomData;
    pub use secp256k1_r::Error;
    pub use self::key::*;

    /// Marker trait for indicating that an instance of `Secp256k1` can be used for signing.
    pub trait Signing {}

    /// Marker trait for indicating that an instance of `Secp256k1` can be used for verification.
    pub trait Verification {}

    /// Represents the set of capabilities needed for signing.
    pub struct SignOnly {}

    /// Represents the set of capabilities needed for verification.
    pub struct VerifyOnly {}

    /// Represents the set of all capabilities.
    pub struct All {}

    impl Signing for SignOnly {}
    impl Signing for All {}

    impl Verification for VerifyOnly {}
    impl Verification for All {}

    /// The secp256k1 engine, used to execute all signature operations
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Secp256k1<C> {
        phantom: PhantomData<C>
    }

    impl<C> Secp256k1<C> {
        pub fn new() -> Self {
            Secp256k1 {
                phantom: PhantomData,
            }
        }
    }

    pub mod constants {
        /// The size (in bytes) of a message
        pub const MESSAGE_SIZE: usize = secp256k1_r::util::MESSAGE_SIZE;

        /// The size (in bytes) of a secret key
        pub const SECRET_KEY_SIZE: usize = secp256k1_r::util::SECRET_KEY_SIZE;

        /// The size (in bytes) of a serialized public key.
        pub const PUBLIC_KEY_SIZE: usize = secp256k1_r::util::COMPRESSED_PUBLIC_KEY_SIZE;

        /// The size (in bytes) of an serialized uncompressed public key
        pub const UNCOMPRESSED_PUBLIC_KEY_SIZE: usize = secp256k1_r::util::FULL_PUBLIC_KEY_SIZE;

        /// The maximum size of a signature
        pub const MAX_SIGNATURE_SIZE: usize = secp256k1_r::util::DER_MAX_SIGNATURE_SIZE;

        /// The maximum size of a compact signature
        pub const COMPACT_SIGNATURE_SIZE: usize = secp256k1_r::util::SIGNATURE_SIZE;

        /// The order of the secp256k1 curve
        pub const CURVE_ORDER: [u8; 32] = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b,
            0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41
        ];

        /// The X coordinate of the generator
        pub const GENERATOR_X: [u8; 32] = [
            0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac,
            0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
            0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9,
            0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98
        ];

        /// The Y coordinate of the generator
        pub const GENERATOR_Y: [u8; 32] = [
            0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65,
            0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11, 0x08, 0xa8,
            0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19,
            0x9c, 0x47, 0xd0, 0x8f, 0xfb, 0x10, 0xd4, 0xb8
        ];
    }

    pub mod ecdh {
        use std::ops;
        use super::key::{SecretKey, PublicKey};

        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct SharedSecret(secp256k1_r::SharedSecret);

        impl SharedSecret {
            pub fn new(point: &PublicKey, scalar: &SecretKey) -> SharedSecret {
                SharedSecret(secp256k1_r::SharedSecret::new(&point.clone().into(), &scalar.clone().into()).unwrap())
            }
        }

        impl ops::Index<usize> for SharedSecret {
            type Output = u8;

            #[inline]
            fn index(&self, index: usize) -> &u8 {
                &self.0.as_ref()[index]
            }
        }

        impl ops::Index<ops::Range<usize>> for SharedSecret {
            type Output = [u8];

            #[inline]
            fn index(&self, index: ops::Range<usize>) -> &[u8] {
                &self.0.as_ref()[index]
            }
        }

        impl ops::Index<ops::RangeFrom<usize>> for SharedSecret {
            type Output = [u8];

            #[inline]
            fn index(&self, index: ops::RangeFrom<usize>) -> &[u8] {
                &self.0.as_ref()[index.start..]
            }
        }

        impl ops::Index<ops::RangeFull> for SharedSecret {
            type Output = [u8];

            #[inline]
            fn index(&self, _: ops::RangeFull) -> &[u8] {
                &self.0.as_ref()[..]
            }
        }

    }

    pub mod key {
        use std::{fmt, ops};
        use super::{Error, Secp256k1};
        use crate::{Signing, Verification};

        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        pub struct SecretKey(pub(crate) [u8; secp256k1_r::util::SECRET_KEY_SIZE]);

        impl From<SecretKey> for secp256k1_r::SecretKey {
            fn from(v: SecretKey) -> Self {
                secp256k1_r::SecretKey::parse(&v.0).unwrap()
            }
        }

        impl fmt::Display for SecretKey {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                for ch in &self.0[..] {
                    write!(f, "{:02x}", *ch)?;
                }
                Ok(())
            }
        }

        impl ops::Index<usize> for SecretKey {
            type Output = u8;

            #[inline]
            fn index(&self, index: usize) -> &u8 {
                &self.0[index]
            }
        }

        impl ops::Index<ops::Range<usize>> for SecretKey {
            type Output = [u8];

            #[inline]
            fn index(&self, index: ops::Range<usize>) -> &[u8] {
                &self.0[index]
            }
        }

        impl ops::Index<ops::RangeFrom<usize>> for SecretKey {
            type Output = [u8];

            #[inline]
            fn index(&self, index: ops::RangeFrom<usize>) -> &[u8] {
                &self.0[index.start..]
            }
        }

        impl ops::Index<ops::RangeTo<usize>> for SecretKey {
            type Output = [u8];

            #[inline]
            fn index(&self, index: ops::RangeTo<usize>) -> &[u8] {
                &self.0[..index.end]
            }
        }

        impl ops::Index<ops::RangeFull> for SecretKey {
            type Output = [u8];

            #[inline]
            fn index(&self, _: ops::RangeFull) -> &[u8] {
                &self.0[..]
            }
        }

        impl SecretKey {
            #[inline]
            pub fn from_slice(data: &[u8]) -> Result<SecretKey, Error> {
                if data.len() == super::constants::SECRET_KEY_SIZE {
                    let mut a = [0; 32];
                    a.copy_from_slice(data);
                    Ok(SecretKey(a))
                } else {
                    Err(Error::InvalidSecretKey)
                }
            }

            pub fn add_assign(&mut self, other: &[u8]) -> Result<(), Error> {
                let mut a: secp256k1_r::SecretKey = self.clone().into();
                let b = secp256k1_r::SecretKey::parse_slice(other)?;
                a.tweak_add_assign(&b)?;
                self.0 = a.serialize();
                Ok(())
            }

            pub fn mul_assign(&mut self, other: &[u8]) -> Result<(), Error> {
                let mut a: secp256k1_r::SecretKey = self.clone().into();
                let b = secp256k1_r::SecretKey::parse_slice(other)?;
                a.tweak_mul_assign(&b)?;
                self.0 = a.serialize();
                Ok(())
            }
        }

        #[derive(Copy, Clone)]
        pub struct PublicKey(pub(crate) [u8; secp256k1_r::util::FULL_PUBLIC_KEY_SIZE]);

        impl From<PublicKey> for secp256k1_r::PublicKey {
            fn from(v: PublicKey) -> Self {
                secp256k1_r::PublicKey::parse(&v.0).unwrap()
            }
        }

        impl fmt::Display for PublicKey {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                for ch in &self.0[..] {
                    write!(f, "{:02x}", *ch)?;
                }
                Ok(())
            }
        }

        impl fmt::Debug for PublicKey {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                for ch in &self.0[..] {
                    write!(f, "{:02x}", *ch)?;
                }
                Ok(())
            }
        }

        impl PartialEq for PublicKey {
            fn eq(&self, other: &PublicKey) -> bool {
                self.0.iter().zip(other.0.iter())
                    .fold(true, |r, (a, b)| r && a.eq(b))
            }
        }

        impl Eq for PublicKey {
        }

        impl PublicKey {
            pub fn from_secret_key<C: Signing>(_secp: &Secp256k1<C>, sk: &SecretKey) -> PublicKey {
                let sk = sk.clone().into();
                let pk = secp256k1_r::PublicKey::from_secret_key(&sk);
                PublicKey(pk.serialize())
            }

            pub fn from_slice(data: &[u8]) -> Result<PublicKey, Error> {
                secp256k1_r::PublicKey::parse_slice(data, None)
                    .map(|x| x.serialize())
                    .map(PublicKey)
            }

            pub fn serialize(&self) -> [u8; super::constants::PUBLIC_KEY_SIZE] {
                secp256k1_r::PublicKey::from(self.clone()).serialize_compressed()
            }

            pub fn add_exp_assign<C: Verification>(
                &mut self,
                _secp: &Secp256k1<C>,
                other: &[u8]
            ) -> Result<(), Error> {
                if other.len() != 32 {
                    return Err(Error::InvalidInputLength);
                }

                let mut pk = secp256k1_r::PublicKey::from(self.clone());

                pk.tweak_add_assign(&secp256k1_r::SecretKey::parse_slice(other)?)?;
                self.0.clone_from_slice(&pk.serialize()[..]);
                Ok(())
            }

            pub fn mul_assign<C: Verification>(
                &mut self,
                _secp: &Secp256k1<C>,
                other: &[u8],
            ) -> Result<(), Error> {
                if other.len() != 32 {
                    return Err(Error::InvalidInputLength);
                }

                let mut pk = secp256k1_r::PublicKey::from(self.clone());

                pk.tweak_mul_assign(&secp256k1_r::SecretKey::parse_slice(other)?)?;
                self.0.clone_from_slice(&pk.serialize()[..]);
                Ok(())
            }

        }
    }
}
