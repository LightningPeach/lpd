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
    use std::{fmt, error, ops::Deref};
    pub use self::key::*;
    use core::fmt::Pointer;

    #[derive(Copy, PartialEq, Eq, Clone, Debug)]
    pub enum Error {
        /// Signature failed verification
        IncorrectSignature,
        /// Badly sized message ("messages" are actually fixed-sized digests; see the `MESSAGE_SIZE`
        /// constant)
        InvalidMessage,
        /// Bad public key
        InvalidPublicKey,
        /// Bad signature
        InvalidSignature,
        /// Bad secret key
        InvalidSecretKey,
        /// Bad recovery id
        InvalidRecoveryId,
        /// Invalid tweak for add_*_assign or mul_*_assign
        InvalidTweak,
        ///
        InvalidInputLength,
    }

    impl From<secp256k1_r::Error> for Error {
        fn from(v: secp256k1_r::Error) -> Self {
            use secp256k1_r::Error::*;

            match v {
                InvalidSignature => Error::InvalidSignature,
                InvalidPublicKey => Error::InvalidPublicKey,
                InvalidSecretKey => Error::InvalidSecretKey,
                InvalidRecoveryId => Error::InvalidRecoveryId,
                InvalidMessage => Error::InvalidMessage,
                InvalidInputLength => Error::InvalidInputLength,
                TweakOutOfRange => Error::InvalidTweak,
            }
        }
    }

    impl From<Error> for secp256k1_r::Error {
        fn from(v: Error) -> Self {
            use secp256k1_r::Error::*;

            match v {
                Error::IncorrectSignature => InvalidSignature,
                Error::InvalidSignature => InvalidSignature,
                Error::InvalidPublicKey => InvalidPublicKey,
                Error::InvalidSecretKey => InvalidSecretKey,
                Error::InvalidRecoveryId => InvalidRecoveryId,
                Error::InvalidMessage => InvalidMessage,
                Error::InvalidInputLength => InvalidInputLength,
                Error::InvalidTweak => TweakOutOfRange,
            }
        }
    }

    impl Error {
        fn as_str(&self) -> &str {
            match *self {
                Error::IncorrectSignature => "secp: signature failed verification",
                Error::InvalidMessage => "secp: message was not 32 bytes (do you need to hash?)",
                Error::InvalidPublicKey => "secp: malformed public key",
                Error::InvalidSignature => "secp: malformed signature",
                Error::InvalidSecretKey => "secp: malformed or out-of-range secret key",
                Error::InvalidRecoveryId => "secp: bad recovery id",
                Error::InvalidInputLength => "secp: bad input length",
                Error::InvalidTweak => "secp: bad tweak",
            }
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            f.write_str(self.as_str())
        }
    }

    impl error::Error for Error {
    }

    #[derive(Clone, PartialEq, Eq)]
    pub struct Signature(secp256k1_r::Signature);

    /// A DER serialized Signature
    #[derive(Copy, Clone)]
    pub struct SerializedSignature {
        data: [u8; 72],
        len: usize,
    }

    impl Default for SerializedSignature {
        fn default() -> SerializedSignature {
            SerializedSignature {
                data: [0u8; 72],
                len: 0,
            }
        }
    }

    impl PartialEq for SerializedSignature {
        fn eq(&self, other: &SerializedSignature) -> bool {
            &self.data[..self.len] == &other.data[..other.len]
        }
    }

    impl Eq for SerializedSignature {}

    impl AsRef<[u8]> for SerializedSignature {
        fn as_ref(&self) -> &[u8] {
            &self.data[..self.len]
        }
    }

    impl Deref for SerializedSignature {
        type Target = [u8];
        fn deref(&self) -> &[u8] {
            &self.data[..self.len]
        }
    }

    impl Signature {
        pub fn from_der(data: &[u8]) -> Result<Signature, Error> {
            secp256k1_r::Signature::parse_der(data)
                .map(Signature)
                .map_err(Error::from)
        }

        pub fn serialize_der(&self) -> SerializedSignature {
            let array = self.0.serialize_der();
            let mut data = [0; 72];
            data[0..array.len()].copy_from_slice(array.as_ref());

            SerializedSignature {
                data: data,
                len: array.len(),
            }
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
    pub struct Message([u8; constants::MESSAGE_SIZE]);

    impl Message {
        pub fn from_slice(data: &[u8]) -> Result<Message, Error> {
            if data == [0; constants::MESSAGE_SIZE] {
                return Err(Error::InvalidMessage);
            }

            match data.len() {
                constants::MESSAGE_SIZE => {
                    let mut ret = [0; constants::MESSAGE_SIZE];
                    ret[..].copy_from_slice(data);
                    Ok(Message(ret))
                }
                _ => Err(Error::InvalidMessage)
            }
        }
    }

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
        use std::{{fmt, ops}, cmp::Ordering, hash::{Hash, Hasher}, str};
        use super::{Error, Secp256k1};
        use crate::{Signing, Verification};

        // copy-paste from secp256k1_c
        fn from_hex(hex: &str, target: &mut [u8]) -> Result<usize, ()> {
            if hex.len() % 2 == 1 || hex.len() > target.len() * 2 {
                return Err(());
            }

            let mut b = 0;
            let mut idx = 0;
            for c in hex.bytes() {
                b <<= 4;
                match c {
                    b'A'...b'F' => b |= c - b'A' + 10,
                    b'a'...b'f' => b |= c - b'a' + 10,
                    b'0'...b'9' => b |= c - b'0',
                    _ => return Err(()),
                }
                if (idx & 1) == 1 {
                    target[idx / 2] = b;
                    b = 0;
                }
                idx += 1;
            }
            Ok(idx / 2)
        }

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

        impl PartialOrd for PublicKey {
            fn partial_cmp(&self, other: &PublicKey) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for PublicKey {
            fn cmp(&self, other: &Self) -> Ordering {
                self.0.iter().zip(other.0.iter())
                    .fold(Ordering::Equal, |r, (a, b)| {
                        r.then(a.cmp(b))
                    })
            }
        }

        impl Hash for PublicKey {
            fn hash<H: Hasher>(&self, state: &mut H) {
                state.write(&self.0[..])
            }
        }

        impl str::FromStr for PublicKey {
            type Err = Error;

            fn from_str(s: &str) -> Result<PublicKey, Error> {
                let mut res = [0; super::constants::UNCOMPRESSED_PUBLIC_KEY_SIZE];
                match from_hex(s, &mut res) {
                    Ok(super::constants::PUBLIC_KEY_SIZE) => {
                        PublicKey::from_slice(
                            &res[0..super::constants::PUBLIC_KEY_SIZE]
                        )
                    }
                    Ok(super::constants::UNCOMPRESSED_PUBLIC_KEY_SIZE) => {
                        PublicKey::from_slice(&res)
                    }
                    _ => Err(Error::InvalidPublicKey)
                }
            }
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
                    .map_err(Error::from)
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

                pk.tweak_add_assign(&secp256k1_r::SecretKey::parse_slice(other)?)
                    .map_err(Error::from)?;
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

                pk.tweak_mul_assign(&secp256k1_r::SecretKey::parse_slice(other)?)
                    .map_err(Error::from)?;
                self.0.clone_from_slice(&pk.serialize()[..]);
                Ok(())
            }

            pub fn serialize_uncompressed(&self) -> [u8; super::constants::UNCOMPRESSED_PUBLIC_KEY_SIZE] {
                let pk = secp256k1_r::PublicKey::from(self.clone());

                pk.serialize()
            }
        }
    }
}
