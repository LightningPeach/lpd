#[cfg(feature = "testing")]
#[macro_export]
macro_rules! public_key {
    ($value:expr) => { {
        use secp256k1::PublicKey;
        use secp256k1::Secp256k1;
        PublicKey::from_slice(&Secp256k1::new(), &hex!($value)).unwrap().into()
    } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binformat::BinarySD;
    use secp256k1::Signature;

    #[test]
    fn signature() {
        let v = vec! [
            169u8, 177, 196, 25, 57, 80, 208, 176, 113, 192, 129, 194, 129, 60, 75, 12,
            21, 77, 188, 167, 162, 88, 249, 147, 231, 18, 208, 195, 174, 189, 240, 95,
            66, 108, 150, 147, 28, 77, 128, 69, 220, 78, 55, 45, 9, 120, 107, 254,
            154, 144, 165, 228, 138, 174, 67, 16, 90, 251, 148, 174, 188, 40, 216, 163,
        ];

        let t: Signature = BinarySD::deserialize(&v[..]).unwrap();
        println!("{:?}", t);

    }
}
