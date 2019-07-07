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
    use std::{fmt, error, ops::Deref, str};
    pub use self::key::*;

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

    impl fmt::Debug for Signature {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(self, f)
        }
    }

    impl fmt::Display for Signature {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let sig = self.serialize_der();
            for v in sig.iter() {
                write!(f, "{:02x}", v)?;
            }
            Ok(())
        }
    }

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

    impl str::FromStr for Signature {
        type Err = Error;
        fn from_str(s: &str) -> Result<Signature, Error> {
            let mut res = [0; 72];
            match from_hex(s, &mut res) {
                Ok(x) => Signature::from_der(&res[0..x]),
                _ => Err(Error::InvalidSignature),
            }
        }
    }

    impl Signature {
        /// Converts a DER-encoded byte slice to a signature
        pub fn from_der(data: &[u8]) -> Result<Signature, Error> {
            secp256k1_r::Signature::parse_der(data)
                .map(Signature)
                .map_err(Error::from)
        }

        /// Converts a "lax DER"-encoded byte slice to a signature. This is basically
        /// only useful for validating signatures in the Bitcoin blockchain from before
        /// 2016. It should never be used in new applications. This library does not
        /// support serializing to this "format"
        pub fn from_der_lax(data: &[u8]) -> Result<Signature, Error> {
            secp256k1_r::Signature::parse_der_lax(data)
                .map(Signature)
                .map_err(Error::from)
        }

        /// Converts a 64-byte compact-encoded byte slice to a signature
        pub fn from_compact(data: &[u8]) -> Result<Signature, Error> {
            if data.len() != 64 {
                return Err(Error::InvalidSignature)
            }

            secp256k1_r::Signature::parse_slice(data)
                .map(Signature)
                .map_err(Error::from)
        }

        /// Serializes the signature in DER format
        pub fn serialize_der(&self) -> SerializedSignature {
            let array = self.0.serialize_der();
            let mut data = [0; 72];
            data[0..array.len()].copy_from_slice(array.as_ref());

            SerializedSignature {
                data: data,
                len: array.len(),
            }
        }

        /// Serializes the signature in compact format
        pub fn serialize_compact(&self) -> [u8; 64] {
            self.0.serialize()
        }

        /// Normalizes a signature to a "low S" form. In ECDSA, signatures are
        /// of the form (r, s) where r and s are numbers lying in some finite
        /// field. The verification equation will pass for (r, s) iff it passes
        /// for (r, -s), so it is possible to ``modify'' signatures in transit
        /// by flipping the sign of s. This does not constitute a forgery since
        /// the signed message still cannot be changed, but for some applications,
        /// changing even the signature itself can be a problem. Such applications
        /// require a "strong signature". It is believed that ECDSA is a strong
        /// signature except for this ambiguity in the sign of s, so to accommodate
        /// these applications libsecp256k1 will only accept signatures for which
        /// s is in the lower half of the field range. This eliminates the
        /// ambiguity.
        ///
        /// However, for some systems, signatures with high s-values are considered
        /// valid. (For example, parsing the historic Bitcoin blockchain requires
        /// this.) For these applications we provide this normalization function,
        /// which ensures that the s value lies in the lower half of its range.
        pub fn normalize_s(&mut self) {
            self.0.normalize_s()
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
        /// (Re)randomizes the Secp256k1 context for cheap sidechannel resistance;
        /// see comment in libsecp256k1 commit d2275795f by Gregory Maxwell. Requires
        /// compilation with "rand" feature.
        #[cfg(any(test, feature = "rand"))]
        pub fn randomize<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) {
            let _ = rng;
        }
    }

    impl Secp256k1<All> {
        pub fn new() -> Self {
            Secp256k1 {
                phantom: PhantomData,
            }
        }
    }

    impl Default for Secp256k1<All> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Secp256k1<SignOnly> {
        /// Creates a new Secp256k1 context that can only be used for signing
        pub fn signing_only() -> Secp256k1<SignOnly> {
            Secp256k1 {
                phantom: PhantomData,
            }
        }
    }

    impl Secp256k1<VerifyOnly> {
        /// Creates a new Secp256k1 context that can only be used for verification
        pub fn verification_only() -> Secp256k1<VerifyOnly> {
            Secp256k1 {
                phantom: PhantomData,
            }
        }
    }

    impl<C: Signing> Secp256k1<C> {
        /// Constructs a signature for `msg` using the secret key `sk` and RFC6979 nonce
        /// Requires a signing-capable context.
        pub fn sign(&self, msg: &Message, sk: &key::SecretKey) -> Signature {
            let sk: secp256k1_r::SecretKey = sk.clone().into();
            let message = secp256k1_r::Message::parse(&msg.0);
            Signature(secp256k1_r::sign(&message, &sk).unwrap().0)
        }

        #[cfg(any(test, feature = "rand"))]
        pub fn generate_keypair<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> (key::SecretKey, key::PublicKey) {
            let sk = key::SecretKey::new(rng);
            let pk = key::PublicKey::from_secret_key(self, &sk);
            (sk, pk)
        }
    }

    impl<C: Verification> Secp256k1<C> {
        /// Checks that `sig` is a valid ECDSA signature for `msg` using the public
        /// key `pubkey`. Returns `Ok(true)` on success. Note that this function cannot
        /// be used for Bitcoin consensus checking since there may exist signatures
        /// which OpenSSL would verify but not libsecp256k1, or vice-versa. Requires a
        /// verify-capable context.
        #[inline]
        pub fn verify(&self, msg: &Message, sig: &Signature, pk: &key::PublicKey) -> Result<(), Error> {
            let message = secp256k1_r::Message::parse(&msg.0);
            let pk = secp256k1_r::PublicKey::from(pk.clone());
            if secp256k1_r::verify(&message, &sig.0, &pk) {
                Ok(())
            } else {
                Err(Error::IncorrectSignature)
            }
        }
    }

    // copy-paste from secp256k1_c
    pub fn from_hex(hex: &str, target: &mut [u8]) -> Result<usize, ()> {
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

        #[cfg(test)]
        mod tests {
            use wasm_bindgen_test::*;

            use rand::thread_rng;
            use super::SharedSecret;
            use super::super::Secp256k1;

            #[wasm_bindgen_test]
            fn ecdh() {
                let s = Secp256k1::signing_only();
                let (sk1, pk1) = s.generate_keypair(&mut thread_rng());
                let (sk2, pk2) = s.generate_keypair(&mut thread_rng());

                let sec1 = SharedSecret::new(&pk1, &sk2);
                let sec2 = SharedSecret::new(&pk2, &sk1);
                let sec_odd = SharedSecret::new(&pk1, &sk1);
                assert_eq!(sec1, sec2);
                assert_ne!(sec_odd, sec2);
            }
        }
    }

    pub mod key {
        use std::{{fmt, ops}, cmp::Ordering, hash::{Hash, Hasher}, str};
        use super::{Error, Secp256k1, from_hex};
        use crate::{Signing, Verification};

        #[derive(Copy, Clone, Eq, PartialEq)]
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

        impl fmt::Debug for SecretKey {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "SecretKey(")?;
                for ch in &self.0[..] {
                    write!(f, "{:02x}", *ch)?;
                }
                write!(f, ")")?;
                Ok(())
            }
        }

        impl str::FromStr for SecretKey {
            type Err = Error;
            fn from_str(s: &str) -> Result<SecretKey, Error> {
                let mut res = [0; super::constants::SECRET_KEY_SIZE];
                match from_hex(s, &mut res) {
                    Ok(super::constants::SECRET_KEY_SIZE) => Ok(SecretKey(res)),
                    _ => Err(Error::InvalidSecretKey)
                }
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
            #[cfg(any(test, feature = "rand"))]
            pub fn new<R: rand::Rng + ?Sized>(rng: &mut R) -> SecretKey {
                let mut data = [0u8; 32];
                loop {
                    rng.fill_bytes(&mut data);
                    if secp256k1_r::SecretKey::parse(&data).is_ok() {
                        break
                    }
                }
                SecretKey(data)
            }

            #[inline]
            pub fn from_slice(data: &[u8]) -> Result<SecretKey, Error> {
                if data.len() == super::constants::SECRET_KEY_SIZE {
                    let mut a = [0; 32];
                    a.copy_from_slice(data);
                    let sk = SecretKey(a);
                    let _sk = secp256k1_r::SecretKey::parse(&a).map_err(Error::from)?;
                    Ok(sk)
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
                for ch in &self.serialize()[..] {
                    write!(f, "{:02x}", *ch)?;
                }
                Ok(())
            }
        }

        impl fmt::Debug for PublicKey {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                for ch in &self.serialize()[..] {
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
                    .map_err(|e| {
                        match e {
                            secp256k1_r::Error::InvalidInputLength => Error::InvalidPublicKey,
                            _ => e.into(),
                        }
                    })
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

            pub fn combine(&self, other: &PublicKey) -> Result<PublicKey, Error> {
                let pk = secp256k1_r::PublicKey::from(self.clone());
                let other = secp256k1_r::PublicKey::from(other.clone());

                Ok(PublicKey(secp256k1_r::PublicKey::combine(&[pk, other])?.serialize()))
            }
        }

        #[cfg(test)]
        mod test {
            use wasm_bindgen_test::*;

            use super::super::Secp256k1;
            use super::super::from_hex;
            use super::super::Error::{InvalidPublicKey, InvalidSecretKey};
            use super::{PublicKey, SecretKey};
            use super::super::constants;

            use rand::{Error, ErrorKind, RngCore, thread_rng};
            use rand_core::impls;
            use std::iter;
            use std::str::FromStr;

            macro_rules! hex {
                ($hex:expr) => ({
                    let mut result = vec![0; $hex.len() / 2];
                    from_hex($hex, &mut result).expect("valid hex string");
                    result
                });
            }

            #[wasm_bindgen_test]
            fn skey_from_slice() {
                let sk = SecretKey::from_slice(&[1; 31]);
                assert_eq!(sk, Err(InvalidSecretKey));

                let sk = SecretKey::from_slice(&[1; 32]);
                assert!(sk.is_ok());
            }

            #[wasm_bindgen_test]
            fn pubkey_from_slice() {
                assert_eq!(PublicKey::from_slice(&[]), Err(InvalidPublicKey));
                assert_eq!(PublicKey::from_slice(&[1, 2, 3]), Err(InvalidPublicKey));

                let uncompressed = PublicKey::from_slice(&[4, 54, 57, 149, 239, 162, 148, 175, 246, 254, 239, 75, 154, 152, 10, 82, 234, 224, 85, 220, 40, 100, 57, 121, 30, 162, 94, 156, 135, 67, 74, 49, 179, 57, 236, 53, 162, 124, 149, 144, 168, 77, 74, 30, 72, 211, 229, 110, 111, 55, 96, 193, 86, 227, 183, 152, 195, 155, 51, 247, 123, 113, 60, 228, 188]);
                assert!(uncompressed.is_ok());

                let compressed = PublicKey::from_slice(&[3, 23, 183, 225, 206, 31, 159, 148, 195, 42, 67, 115, 146, 41, 248, 140, 11, 3, 51, 41, 111, 180, 110, 143, 114, 134, 88, 73, 198, 174, 52, 184, 78]);
                assert!(compressed.is_ok());
            }

            #[wasm_bindgen_test]
            fn keypair_slice_round_trip() {
                let s = Secp256k1::new();

                let (sk1, pk1) = s.generate_keypair(&mut thread_rng());
                assert_eq!(SecretKey::from_slice(&sk1[..]), Ok(sk1));
                assert_eq!(PublicKey::from_slice(&pk1.serialize()[..]), Ok(pk1));
                assert_eq!(PublicKey::from_slice(&pk1.serialize_uncompressed()[..]), Ok(pk1));
            }

            #[wasm_bindgen_test]
            fn invalid_secret_key() {
                // Zero
                assert_eq!(SecretKey::from_slice(&[0; 32]), Err(InvalidSecretKey));
                // -1
                assert_eq!(SecretKey::from_slice(&[0xff; 32]), Err(InvalidSecretKey));
                // Top of range
                assert!(SecretKey::from_slice(&[
                    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
                    0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
                    0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x40,
                ]).is_ok());
                // One past top of range
                assert!(SecretKey::from_slice(&[
                    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
                    0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
                    0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
                ]).is_err());
            }

            #[wasm_bindgen_test]
            fn test_out_of_range() {
                struct BadRng(u8);
                impl RngCore for BadRng {
                    fn next_u32(&mut self) -> u32 { unimplemented!() }
                    fn next_u64(&mut self) -> u64 { unimplemented!() }
                    // This will set a secret key to a little over the
                    // group order, then decrement with repeated calls
                    // until it returns a valid key
                    fn fill_bytes(&mut self, data: &mut [u8]) {
                        let group_order: [u8; 32] = [
                            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
                            0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b,
                            0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41];
                        assert_eq!(data.len(), 32);
                        data.copy_from_slice(&group_order[..]);
                        data[31] = self.0;
                        self.0 -= 1;
                    }
                    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
                        Ok(self.fill_bytes(dest))
                    }
                }

                let s = Secp256k1::new();
                s.generate_keypair(&mut BadRng(0xff));
            }

            #[wasm_bindgen_test]
            fn test_pubkey_from_bad_slice() {
                // Bad sizes
                assert_eq!(
                    PublicKey::from_slice(&[0; constants::PUBLIC_KEY_SIZE - 1]),
                    Err(InvalidPublicKey)
                );
                assert_eq!(
                    PublicKey::from_slice(&[0; constants::PUBLIC_KEY_SIZE + 1]),
                    Err(InvalidPublicKey)
                );
                assert_eq!(
                    PublicKey::from_slice(&[0; constants::UNCOMPRESSED_PUBLIC_KEY_SIZE - 1]),
                    Err(InvalidPublicKey)
                );
                assert_eq!(
                    PublicKey::from_slice(&[0; constants::UNCOMPRESSED_PUBLIC_KEY_SIZE + 1]),
                    Err(InvalidPublicKey)
                );

                // Bad parse
                assert_eq!(
                    PublicKey::from_slice(&[0xff; constants::UNCOMPRESSED_PUBLIC_KEY_SIZE]),
                    Err(InvalidPublicKey)
                );
                assert_eq!(
                    PublicKey::from_slice(&[0x55; constants::PUBLIC_KEY_SIZE]),
                    Err(InvalidPublicKey)
                );
            }

            #[wasm_bindgen_test]
            fn test_debug_output() {
                struct DumbRng(u32);
                impl RngCore for DumbRng {
                    fn next_u32(&mut self) -> u32 {
                        self.0 = self.0.wrapping_add(1);
                        self.0
                    }

                    fn next_u64(&mut self) -> u64 {
                        self.next_u32() as u64
                    }

                    fn fill_bytes(&mut self, dest: &mut [u8]) {
                        impls::fill_bytes_via_next(self, dest);
                    }

                    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), Error> {
                        Err(Error::new(ErrorKind::Unavailable, "not implemented"))
                    }
                }

                let s = Secp256k1::new();
                let (sk, _) = s.generate_keypair(&mut DumbRng(0));

                assert_eq!(&format!("{:?}", sk), "SecretKey(0100000000000000020000000000000003000000000000000400000000000000)");
            }

            #[wasm_bindgen_test]
            fn test_display_output() {
                static SK_BYTES: [u8; 32] = [
                    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                    0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00,
                    0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x63,
                ];

                let s = Secp256k1::signing_only();
                let sk = SecretKey::from_slice(&SK_BYTES).expect("sk");
                let pk = PublicKey::from_secret_key(&s, &sk);

                assert_eq!(
                    sk.to_string(),
                    "01010101010101010001020304050607ffff0000ffff00006363636363636363"
                );
                assert_eq!(
                    SecretKey::from_str("01010101010101010001020304050607ffff0000ffff00006363636363636363").unwrap(),
                    sk
                );
                assert_eq!(
                    pk.to_string(),
                    "0218845781f631c48f1c9709e23092067d06837f30aa0cd0544ac887fe91ddd166"
                );
                assert_eq!(
                    PublicKey::from_str("0218845781f631c48f1c9709e23092067d06837f30aa0cd0544ac887fe91ddd166").unwrap(),
                    pk
                );
                assert_eq!(
                    PublicKey::from_str("04\
                18845781f631c48f1c9709e23092067d06837f30aa0cd0544ac887fe91ddd166\
                84B84DB303A340CD7D6823EE88174747D12A67D2F8F2F9BA40846EE5EE7A44F6"
                    ).unwrap(),
                    pk
                );

                assert!(SecretKey::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").is_err());
                assert!(SecretKey::from_str("01010101010101010001020304050607ffff0000ffff0000636363636363636363").is_err());
                assert!(SecretKey::from_str("01010101010101010001020304050607ffff0000ffff0000636363636363636").is_err());
                assert!(SecretKey::from_str("01010101010101010001020304050607ffff0000ffff000063636363636363").is_err());
                assert!(SecretKey::from_str("01010101010101010001020304050607ffff0000ffff000063636363636363xx").is_err());
                assert!(PublicKey::from_str("0300000000000000000000000000000000000000000000000000000000000000000").is_err());
                assert!(PublicKey::from_str("0218845781f631c48f1c9709e23092067d06837f30aa0cd0544ac887fe91ddd16601").is_err());
                assert!(PublicKey::from_str("0218845781f631c48f1c9709e23092067d06837f30aa0cd0544ac887fe91ddd16").is_err());
                assert!(PublicKey::from_str("0218845781f631c48f1c9709e23092067d06837f30aa0cd0544ac887fe91ddd1").is_err());
                assert!(PublicKey::from_str("xx0218845781f631c48f1c9709e23092067d06837f30aa0cd0544ac887fe91ddd1").is_err());

                let long_str: String = iter::repeat('a').take(1024 * 1024).collect();
                assert!(SecretKey::from_str(&long_str).is_err());
                assert!(PublicKey::from_str(&long_str).is_err());
            }

            #[wasm_bindgen_test]
            fn test_pubkey_serialize() {
                struct DumbRng(u32);
                impl RngCore for DumbRng {
                    fn next_u32(&mut self) -> u32 {
                        self.0 = self.0.wrapping_add(1);
                        self.0
                    }
                    fn next_u64(&mut self) -> u64 {
                        self.next_u32() as u64
                    }

                    fn fill_bytes(&mut self, dest: &mut [u8]) {
                        impls::fill_bytes_via_next(self, dest);
                    }

                    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), Error> {
                        Err(Error::new(ErrorKind::Unavailable, "not implemented"))
                    }
                }

                let s = Secp256k1::new();
                let (_, pk1) = s.generate_keypair(&mut DumbRng(0));
                assert_eq!(&pk1.serialize_uncompressed()[..],
                           &[4, 124, 121, 49, 14, 253, 63, 197, 50, 39, 194, 107, 17, 193, 219, 108, 154, 126, 9, 181, 248, 2, 12, 149, 233, 198, 71, 149, 134, 250, 184, 154, 229, 185, 28, 165, 110, 27, 3, 162, 126, 238, 167, 157, 242, 221, 76, 251, 237, 34, 231, 72, 39, 245, 3, 191, 64, 111, 170, 117, 103, 82, 28, 102, 163][..]);
                assert_eq!(&pk1.serialize()[..],
                           &[3, 124, 121, 49, 14, 253, 63, 197, 50, 39, 194, 107, 17, 193, 219, 108, 154, 126, 9, 181, 248, 2, 12, 149, 233, 198, 71, 149, 134, 250, 184, 154, 229][..]);
            }

            #[wasm_bindgen_test]
            fn test_addition() {
                let s = Secp256k1::new();

                let (mut sk1, mut pk1) = s.generate_keypair(&mut thread_rng());
                let (mut sk2, mut pk2) = s.generate_keypair(&mut thread_rng());

                assert_eq!(PublicKey::from_secret_key(&s, &sk1), pk1);
                assert!(sk1.add_assign(&sk2[..]).is_ok());
                assert!(pk1.add_exp_assign(&s, &sk2[..]).is_ok());
                assert_eq!(PublicKey::from_secret_key(&s, &sk1), pk1);

                assert_eq!(PublicKey::from_secret_key(&s, &sk2), pk2);
                assert!(sk2.add_assign(&sk1[..]).is_ok());
                assert!(pk2.add_exp_assign(&s, &sk1[..]).is_ok());
                assert_eq!(PublicKey::from_secret_key(&s, &sk2), pk2);
            }

            #[wasm_bindgen_test]
            fn test_multiplication() {
                let s = Secp256k1::new();

                let (mut sk1, mut pk1) = s.generate_keypair(&mut thread_rng());
                let (mut sk2, mut pk2) = s.generate_keypair(&mut thread_rng());

                assert_eq!(PublicKey::from_secret_key(&s, &sk1), pk1);
                assert!(sk1.mul_assign(&sk2[..]).is_ok());
                assert!(pk1.mul_assign(&s, &sk2[..]).is_ok());
                assert_eq!(PublicKey::from_secret_key(&s, &sk1), pk1);

                assert_eq!(PublicKey::from_secret_key(&s, &sk2), pk2);
                assert!(sk2.mul_assign(&sk1[..]).is_ok());
                assert!(pk2.mul_assign(&s, &sk1[..]).is_ok());
                assert_eq!(PublicKey::from_secret_key(&s, &sk2), pk2);
            }

            #[wasm_bindgen_test]
            fn pubkey_hash() {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                use std::collections::HashSet;

                fn hash<T: Hash>(t: &T) -> u64 {
                    let mut s = DefaultHasher::new();
                    t.hash(&mut s);
                    s.finish()
                }

                let s = Secp256k1::new();
                let mut set = HashSet::new();
                const COUNT : usize = 1024;
                let count = (0..COUNT).map(|_| {
                    let (_, pk) = s.generate_keypair(&mut thread_rng());
                    let hash = hash(&pk);
                    assert!(!set.contains(&hash));
                    set.insert(hash);
                }).count();
                assert_eq!(count, COUNT);
            }

            #[wasm_bindgen_test]
            fn pubkey_combine() {
                let compressed1 = PublicKey::from_slice(
                    &hex!("0241cc121c419921942add6db6482fb36243faf83317c866d2a28d8c6d7089f7ba"),
                ).unwrap();
                let compressed2 = PublicKey::from_slice(
                    &hex!("02e6642fd69bd211f93f7f1f36ca51a26a5290eb2dd1b0d8279a87bb0d480c8443"),
                ).unwrap();
                let exp_sum = PublicKey::from_slice(
                    &hex!("0384526253c27c7aef56c7b71a5cd25bebb66dddda437826defc5b2568bde81f07"),
                ).unwrap();

                let sum1 = compressed1.combine(&compressed2);
                assert!(sum1.is_ok());
                let sum2 = compressed2.combine(&compressed1);
                assert!(sum2.is_ok());
                assert_eq!(sum1, sum2);
                assert_eq!(sum1.unwrap(), exp_sum);
            }

            #[wasm_bindgen_test]
            fn pubkey_equal() {
                let pk1 = PublicKey::from_slice(
                    &hex!("0241cc121c419921942add6db6482fb36243faf83317c866d2a28d8c6d7089f7ba"),
                ).unwrap();
                let pk2 = pk1.clone();
                let pk3 = PublicKey::from_slice(
                    &hex!("02e6642fd69bd211f93f7f1f36ca51a26a5290eb2dd1b0d8279a87bb0d480c8443"),
                ).unwrap();

                assert_eq!(pk1, pk2);
                assert!(pk1 <= pk2);
                assert!(pk2 <= pk1);
                assert!(!(pk2 < pk1));
                assert!(!(pk1 < pk2));

                // TODO: comparison is not compatible with secp256k1
                assert!(pk3 > pk1);
                assert!(pk1 < pk3);
                assert!(pk3 >= pk1);
                assert!(pk1 <= pk3);
            }

            #[cfg(feature = "serde")]
            #[wasm_bindgen_test]
            fn test_signature_serde() {
                use serde_test::{Configure, Token, assert_tokens};
                static SK_BYTES: [u8; 32] = [
                    1, 1, 1, 1, 1, 1, 1, 1,
                    0, 1, 2, 3, 4, 5, 6, 7,
                    0xff, 0xff, 0, 0, 0xff, 0xff, 0, 0,
                    99, 99, 99, 99, 99, 99, 99, 99
                ];
                static SK_STR: &'static str = "\
                    01010101010101010001020304050607ffff0000ffff00006363636363636363\
                ";
                static PK_BYTES: [u8; 33] = [
                    0x02,
                    0x18, 0x84, 0x57, 0x81, 0xf6, 0x31, 0xc4, 0x8f,
                    0x1c, 0x97, 0x09, 0xe2, 0x30, 0x92, 0x06, 0x7d,
                    0x06, 0x83, 0x7f, 0x30, 0xaa, 0x0c, 0xd0, 0x54,
                    0x4a, 0xc8, 0x87, 0xfe, 0x91, 0xdd, 0xd1, 0x66,
                ];

                let s = Secp256k1::new();

                let sk = SecretKey::from_slice(&SK_BYTES).unwrap();
                let pk = PublicKey::from_secret_key(&s, &sk);

                assert_tokens(&sk.compact(), &[Token::BorrowedBytes(&SK_BYTES[..])]);
                assert_tokens(&sk.readable(), &[Token::BorrowedStr(SK_STR)]);
                assert_tokens(&pk, &[Token::BorrowedBytes(&PK_BYTES[..])]);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use wasm_bindgen_test::*;

        use rand::{RngCore, thread_rng};
        use std::str::FromStr;

        use super::key::{SecretKey, PublicKey};
        use super::from_hex;
        use super::constants;
        use super::{Secp256k1, Signature, Message};
        use super::Error::{InvalidMessage, IncorrectSignature, InvalidSignature};

        macro_rules! hex {
            ($hex:expr) => ({
                let mut result = vec![0; $hex.len() / 2];
                from_hex($hex, &mut result).expect("valid hex string");
                result
            });
        }

        #[wasm_bindgen_test]
        fn capabilities() {
            let sign = Secp256k1::signing_only();
            let vrfy = Secp256k1::verification_only();
            let full = Secp256k1::new();

            let mut msg = [0u8; 32];
            thread_rng().fill_bytes(&mut msg);
            let msg = Message::from_slice(&msg).unwrap();

            // Try key generation
            let (sk, pk) = full.generate_keypair(&mut thread_rng());

            // Try signing
            assert_eq!(sign.sign(&msg, &sk), full.sign(&msg, &sk));
            let sig = full.sign(&msg, &sk);

            // Try verifying
            assert!(vrfy.verify(&msg, &sig, &pk).is_ok());
            assert!(full.verify(&msg, &sig, &pk).is_ok());

            // Check that we can produce keys from slices with no precomputation
            let (pk_slice, sk_slice) = (&pk.serialize(), &sk[..]);
            let new_pk = PublicKey::from_slice(pk_slice).unwrap();
            let new_sk = SecretKey::from_slice(sk_slice).unwrap();
            assert_eq!(sk, new_sk);
            assert_eq!(pk, new_pk);
        }

        #[wasm_bindgen_test]
        fn signature_serialize_roundtrip() {
            let mut s = Secp256k1::new();
            s.randomize(&mut thread_rng());

            let mut msg = [0; 32];
            for _ in 0..100 {
                thread_rng().fill_bytes(&mut msg);
                let msg = Message::from_slice(&msg).unwrap();

                let (sk, _) = s.generate_keypair(&mut thread_rng());
                let sig1 = s.sign(&msg, &sk);
                let der = sig1.serialize_der();
                let sig2 = Signature::from_der(&der[..]).unwrap();
                assert_eq!(sig1, sig2);

                let compact = sig1.serialize_compact();
                let sig2 = Signature::from_compact(&compact[..]).unwrap();
                assert_eq!(sig1, sig2);

                assert!(Signature::from_compact(&der[..]).is_err());
                assert!(Signature::from_compact(&compact[0..4]).is_err());
                assert!(Signature::from_der(&compact[..]).is_err());
                assert!(Signature::from_der(&der[0..4]).is_err());
            }
        }

        #[wasm_bindgen_test]
        fn signature_display() {
            let hex_str = "3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45";
            let byte_str = hex!(hex_str);

            assert_eq!(
                Signature::from_der(&byte_str).expect("byte str decode"),
                Signature::from_str(&hex_str).expect("byte str decode")
            );

            let sig = Signature::from_str(&hex_str).expect("byte str decode");
            assert_eq!(&sig.to_string(), hex_str);
            assert_eq!(&format!("{:?}", sig), hex_str);

            assert!(Signature::from_str(
                "3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab4"
            ).is_err());
            assert!(Signature::from_str(
                "3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab"
            ).is_err());
            assert!(Signature::from_str(
                "3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eabxx"
            ).is_err());
            assert!(Signature::from_str(
                "3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45\
             72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45"
            ).is_err());

            // 71 byte signature
            let hex_str = "30450221009d0bad576719d32ae76bedb34c774866673cbde3f4e12951555c9408e6ce774b02202876e7102f204f6bfee26c967c3926ce702cf97d4b010062e193f763190f6776";
            let sig = Signature::from_str(&hex_str).expect("byte str decode");
            assert_eq!(&format!("{}", sig), hex_str);
        }

        macro_rules! check_lax_sig(
            ($hex:expr) => ({
                let sig = hex!($hex);
                assert!(Signature::from_der_lax(&sig[..]).is_ok());
            })
        );

        #[wasm_bindgen_test]
        fn signature_lax_der() {
            check_lax_sig!("304402204c2dd8a9b6f8d425fcd8ee9a20ac73b619906a6367eac6cb93e70375225ec0160220356878eff111ff3663d7e6bf08947f94443845e0dcc54961664d922f7660b80c");
            check_lax_sig!("304402202ea9d51c7173b1d96d331bd41b3d1b4e78e66148e64ed5992abd6ca66290321c0220628c47517e049b3e41509e9d71e480a0cdc766f8cdec265ef0017711c1b5336f");
            check_lax_sig!("3045022100bf8e050c85ffa1c313108ad8c482c4849027937916374617af3f2e9a881861c9022023f65814222cab09d5ec41032ce9c72ca96a5676020736614de7b78a4e55325a");
            check_lax_sig!("3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45");
            check_lax_sig!("3046022100eaa5f90483eb20224616775891397d47efa64c68b969db1dacb1c30acdfc50aa022100cf9903bbefb1c8000cf482b0aeeb5af19287af20bd794de11d82716f9bae3db1");
            check_lax_sig!("3045022047d512bc85842ac463ca3b669b62666ab8672ee60725b6c06759e476cebdc6c102210083805e93bd941770109bcc797784a71db9e48913f702c56e60b1c3e2ff379a60");
            check_lax_sig!("3044022023ee4e95151b2fbbb08a72f35babe02830d14d54bd7ed1320e4751751d1baa4802206235245254f58fd1be6ff19ca291817da76da65c2f6d81d654b5185dd86b8acf");
        }

        #[wasm_bindgen_test]
        fn sign_and_verify() {
            let mut s = Secp256k1::new();
            s.randomize(&mut thread_rng());

            let mut msg = [0; 32];
            for _ in 0..100 {
                thread_rng().fill_bytes(&mut msg);
                let msg = Message::from_slice(&msg).unwrap();

                let (sk, pk) = s.generate_keypair(&mut thread_rng());
                let sig = s.sign(&msg, &sk);
                assert_eq!(s.verify(&msg, &sig, &pk), Ok(()));
            }
        }

        #[wasm_bindgen_test]
        fn sign_and_verify_extreme() {
            let mut s = Secp256k1::new();
            s.randomize(&mut thread_rng());

            // Wild keys: 1, CURVE_ORDER - 1
            // Wild msgs: 1, CURVE_ORDER - 1
            let mut wild_keys = [[0; 32]; 2];
            let mut wild_msgs = [[0; 32]; 2];

            wild_keys[0][0] = 1;
            wild_msgs[0][0] = 1;

            use constants;
            wild_keys[1][..].copy_from_slice(&constants::CURVE_ORDER[..]);
            wild_msgs[1][..].copy_from_slice(&constants::CURVE_ORDER[..]);

            wild_keys[1][0] -= 1;
            wild_msgs[1][0] -= 1;

            for key in wild_keys.iter().map(|k| SecretKey::from_slice(&k[..]).unwrap()) {
                for msg in wild_msgs.iter().map(|m| Message::from_slice(&m[..]).unwrap()) {
                    let sig = s.sign(&msg, &key);
                    let pk = PublicKey::from_secret_key(&s, &key);
                    assert_eq!(s.verify(&msg, &sig, &pk), Ok(()));
                }
            }
        }

        #[wasm_bindgen_test]
        fn sign_and_verify_fail() {
            let mut s = Secp256k1::new();
            s.randomize(&mut thread_rng());

            let mut msg = [0u8; 32];
            thread_rng().fill_bytes(&mut msg);
            let msg = Message::from_slice(&msg).unwrap();

            let (sk, pk) = s.generate_keypair(&mut thread_rng());

            let sig = s.sign(&msg, &sk);

            let mut msg = [0u8; 32];
            thread_rng().fill_bytes(&mut msg);
            let msg = Message::from_slice(&msg).unwrap();
            assert_eq!(s.verify(&msg, &sig, &pk), Err(IncorrectSignature));
        }

        #[wasm_bindgen_test]
        fn test_bad_slice() {
            assert_eq!(Signature::from_der(&[0; constants::MAX_SIGNATURE_SIZE + 1]),
                       Err(InvalidSignature));
            assert_eq!(Signature::from_der(&[0; constants::MAX_SIGNATURE_SIZE]),
                       Err(InvalidSignature));

            assert_eq!(Message::from_slice(&[0; constants::MESSAGE_SIZE - 1]),
                       Err(InvalidMessage));
            assert_eq!(Message::from_slice(&[0; constants::MESSAGE_SIZE + 1]),
                       Err(InvalidMessage));
            assert_eq!(
                Message::from_slice(&[0; constants::MESSAGE_SIZE]),
                Err(InvalidMessage)
            );
            assert!(Message::from_slice(&[1; constants::MESSAGE_SIZE]).is_ok());
        }

        #[wasm_bindgen_test]
        fn test_low_s() {
            // nb this is a transaction on testnet
            // txid 8ccc87b72d766ab3128f03176bb1c98293f2d1f85ebfaf07b82cc81ea6891fa9
            //      input number 3
            let sig = hex!("3046022100839c1fbc5304de944f697c9f4b1d01d1faeba32d751c0f7acb21ac8a0f436a72022100e89bd46bb3a5a62adc679f659b7ce876d83ee297c7a5587b2011c4fcc72eab45");
            let pk = hex!("031ee99d2b786ab3b0991325f2de8489246a6a3fdb700f6d0511b1d80cf5f4cd43");
            let msg = hex!("a4965ca63b7d8562736ceec36dfa5a11bf426eb65be8ea3f7a49ae363032da0d");

            let secp = Secp256k1::new();
            let mut sig = Signature::from_der(&sig[..]).unwrap();
            let pk = PublicKey::from_slice(&pk[..]).unwrap();
            let msg = Message::from_slice(&msg[..]).unwrap();

            // without normalization we expect this will not fail, because it will normalize internally
            assert_eq!(secp.verify(&msg, &sig, &pk), Ok(()));
            // after normalization it should pass
            sig.normalize_s();
            assert_eq!(secp.verify(&msg, &sig, &pk), Ok(()));
        }

        #[cfg(feature = "serde")]
        #[wasm_bindgen_test]
        fn test_signature_serde() {
            use serde_test::{Configure, Token, assert_tokens};

            let s = Secp256k1::new();

            let msg = Message::from_slice(&[1; 32]).unwrap();
            let sk = SecretKey::from_slice(&[2; 32]).unwrap();
            let sig = s.sign(&msg, &sk);
            static SIG_BYTES: [u8; 71] = [
                48, 69, 2, 33, 0, 157, 11, 173, 87, 103, 25, 211, 42, 231, 107, 237,
                179, 76, 119, 72, 102, 103, 60, 189, 227, 244, 225, 41, 81, 85, 92, 148,
                8, 230, 206, 119, 75, 2, 32, 40, 118, 231, 16, 47, 32, 79, 107, 254,
                226, 108, 150, 124, 57, 38, 206, 112, 44, 249, 125, 75, 1, 0, 98, 225,
                147, 247, 99, 25, 15, 103, 118
            ];
            static SIG_STR: &'static str = "\
                30450221009d0bad576719d32ae76bedb34c774866673cbde3f4e12951555c9408e6ce77\
                4b02202876e7102f204f6bfee26c967c3926ce702cf97d4b010062e193f763190f6776\
            ";

            assert_tokens(&sig.compact(), &[Token::BorrowedBytes(&SIG_BYTES[..])]);
            assert_tokens(&sig.readable(), &[Token::BorrowedStr(SIG_STR)]);
        }
    }
}
