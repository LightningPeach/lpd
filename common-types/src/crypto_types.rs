use std::error::Error;

use dependencies::secp256k1;
use dependencies::hex;

use secp256k1::{Signature, PublicKey};
use binformat::PackSized;

#[cfg(feature = "testing")]
#[macro_export]
macro_rules! public_key {
    ($value:expr) => { {
        use secp256k1::PublicKey;
        use secp256k1::Secp256k1;
        PublicKey::from_slice(&Secp256k1::new(), &hex!($value)).unwrap().into()
    } }
}

#[derive(Clone, Eq, PartialEq)]
pub struct RawPublicKey(pub PublicKey);

impl From<PublicKey> for RawPublicKey {
    fn from(v: PublicKey) -> Self {
        RawPublicKey(v)
    }
}

impl std::fmt::Display for RawPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RawPublicKey({})", self.to_hex())
    }
}

impl std::fmt::Debug for RawPublicKey {
    // use compact format, same as as to_hex()
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RawPublicKey({})", self.to_hex())
    }
}

impl AsRef<PublicKey> for RawPublicKey {
    fn as_ref(&self) -> &PublicKey {
        match self {
            &RawPublicKey(ref i) => i,
        }
    }
}

impl RawPublicKey {
    // assume serialized compressed or not?
    pub fn from_hex(s: &str) -> Result<RawPublicKey, Box<dyn Error>> {
        use std::str::FromStr;
        let pk = PublicKey::from_str(s)
            .map_err(|err| format!("error decoding pubkey: {:?}", err))?;
        Ok(RawPublicKey(pk))
    }

    // assume serialized compressed
    pub fn to_hex(&self) -> String {
        let b = self.0.serialize();
        hex::encode(&b[..])
    }
}

impl PackSized for RawPublicKey {
}

impl serde::Serialize for RawPublicKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use binformat::SerdeRawVec;
        SerdeRawVec(self.0.serialize().to_vec()).serialize(s)
    }
}

impl<'de> serde::Deserialize<'de> for RawPublicKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<RawPublicKey, D::Error> {
        use serde::de::Error;

        let fh: (u8, [u8; 32]) = serde::Deserialize::deserialize(d)?;
        let mut array = [0; 33];
        array[0] = fh.0;
        array[1..].copy_from_slice(&fh.1[..]);
        PublicKey::from_slice(&array[..]).map_err(D::Error::custom).map(RawPublicKey)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RawSignature(pub Signature);

impl From<Signature> for RawSignature {
    fn from(v: Signature) -> Self {
        RawSignature(v)
    }
}

impl AsRef<Signature> for RawSignature {
    fn as_ref(&self) -> &Signature {
        match self {
            &RawSignature(ref i) => i,
        }
    }
}

impl RawSignature {
    // use der encoding
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.serialize_der().to_vec()
    }

    // use der encoding
    pub fn from_bytes(v: &[u8]) -> Result<RawSignature, Box<dyn Error>> {
        let s = Signature::from_der(v)?;
        Ok(RawSignature(s))
    }

    // use der encoding
    pub fn to_hex(&self) -> String {
        let b = self.to_bytes();
        hex::encode(&b[..])
    }

    // use der encoding
    pub fn from_hex(s: &str) -> Result<RawSignature, Box<dyn Error>> {
        let b = hex::decode(s)
            .map_err(|err| format!("cannot decode RawSignature from hex: {:?}", err))?;
        let sig = Signature::from_der(&b[..])
            .map_err(|err| format!("cannot create RawSignature from bytes using der encoding: {:?}", err))?;
        Ok(RawSignature(sig))
    }
}

impl PackSized for RawSignature {
}

impl serde::Serialize for RawSignature {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use binformat::SerdeRawVec;
        SerdeRawVec(self.0.serialize_compact().to_vec()).serialize(s)
    }
}

impl<'de> serde::Deserialize<'de> for RawSignature {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<RawSignature, D::Error> {
        use serde::de::Error;

        let fh: [[u8; 32]; 2] = serde::Deserialize::deserialize(d)?;
        let mut array = [0; 64];
        array[..32].copy_from_slice(&fh[0][..]);
        array[32..].copy_from_slice(&fh[1][..]);
        Signature::from_compact(&array[..]).map_err(D::Error::custom).map(RawSignature)
    }
}

#[cfg(test)]
mod tests {
    use binformat::BinarySD;
    use super::RawSignature;

    #[test]
    fn signature() {
        let v = vec! [
            169u8, 177, 196, 25, 57, 80, 208, 176, 113, 192, 129, 194, 129, 60, 75, 12,
            21, 77, 188, 167, 162, 88, 249, 147, 231, 18, 208, 195, 174, 189, 240, 95,
            66, 108, 150, 147, 28, 77, 128, 69, 220, 78, 55, 45, 9, 120, 107, 254,
            154, 144, 165, 228, 138, 174, 67, 16, 90, 251, 148, 174, 188, 40, 216, 163,
        ];

        let t: RawSignature = BinarySD::deserialize(&v[..]).unwrap();
        println!("{:?}", t);
    }
}
