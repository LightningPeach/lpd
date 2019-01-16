use secp256k1::{SecretKey, Secp256k1, PublicKey};
use super::tools::sha256;

// pubkey = basepoint + SHA256(per_commitment_point || basepoint) * G
pub fn derive_pubkey(base_point: &PublicKey, per_commitment_point: &PublicKey) -> PublicKey {
    let joined = [&per_commitment_point.serialize()[..], &base_point.serialize()[..]].concat();
    let h = sha256(&joined);
    let ctx = Secp256k1::new();
    // TODO(mkl): maybe return error instead of unwrap
    let sk = SecretKey::from_slice(&h).unwrap();
    let pk = PublicKey::from_secret_key(&ctx, &sk);
    let rez = pk.combine(&base_point).unwrap();
    return rez;
}

// privkey = basepoint_secret + SHA256(per_commitment_point || basepoint)
pub fn derive_privkey(base_point_secret: &SecretKey, per_commitment_point: &PublicKey) -> SecretKey {
    let ctx = Secp256k1::new();
    let base_point = PublicKey::from_secret_key(&ctx, base_point_secret);
    let joined = [&per_commitment_point.serialize()[..], &base_point.serialize()[..]].concat();
    let h = sha256(&joined);
    // TODO(mkl): maybe return error instead of unwrap
    let mut sk = base_point_secret.clone();
    sk.add_assign(&h[..]).unwrap();
    return sk
}

// revocation_pubkey = revocation_basepoint * SHA256(revocation_basepoint || per_commitment_point)
//      + per_commitment_point * SHA256(per_commitment_point || revocation_basepoint)
pub fn derive_revocation_pubkey(revocation_base_point: &PublicKey, per_commitment_point: &PublicKey) -> PublicKey {
    let ctx = Secp256k1::new();

    let joined1 = [&revocation_base_point.serialize()[..], &per_commitment_point.serialize()[..]].concat();
    let joined2 = [&per_commitment_point.serialize()[..], &revocation_base_point.serialize()[..]].concat();
    let h1 = sha256(&joined1);
    let h2 = sha256(&joined2);

    let mut pk1 = revocation_base_point.clone();
    pk1.mul_assign(&ctx, &h1[..]).unwrap();
    let mut pk2 = per_commitment_point.clone();
    pk2.mul_assign(&ctx, &h2[..]).unwrap();

    let rez = pk1.combine(&pk2).unwrap();
    return rez;
}

// revocationprivkey = revocation_basepoint_secret * SHA256(revocation_basepoint || per_commitment_point)
//        + per_commitment_secret * SHA256(per_commitment_point || revocation_basepoint)
pub fn derive_revocation_privkey(revocation_base_point_secret: &SecretKey, per_commitment_point_secret: &SecretKey) -> SecretKey {
    let ctx = Secp256k1::new();

    let revocation_base_point = PublicKey::from_secret_key(
        &ctx,
        revocation_base_point_secret
    );

    let per_commitment_point = PublicKey::from_secret_key(
        &ctx,
        per_commitment_point_secret,
    );

    let joined1 = [&revocation_base_point.serialize()[..], &per_commitment_point.serialize()[..]].concat();
    let joined2 = [&per_commitment_point.serialize()[..], &revocation_base_point.serialize()[..]].concat();
    let h1 = sha256(&joined1);
    let h2 = sha256(&joined2);

    // TODO(mkl): maybe return error instead of unwrap
    let mut sk1 = revocation_base_point_secret.clone();
    let mut sk2 = per_commitment_point_secret.clone();

    sk1.mul_assign(&h1[..]).unwrap();
    sk2.mul_assign(&h2[..]).unwrap();

    use std::slice::from_raw_parts;
    sk1.add_assign(unsafe { from_raw_parts(sk2.as_ptr(), 32) }).unwrap();
    return sk1;
}


#[cfg(test)]
mod tests {
    use super::super::tools::{s2pubkey, s2privkey};
    use super::{derive_pubkey, derive_privkey, derive_revocation_pubkey, derive_revocation_privkey};

    #[test]
    fn test_derive_pubkey() {
        // base_point 036d6caac248af96f6afa7f904f550253a0f3ef3f5aa2fe6838a95b216691468e2
        // per_commitment_point: 025f7117a78150fe2ef97db7cfc83bd57b2e2c0d0dd25eaf467a4a1c2a45ce1486
        // resulting pubkey: 0235f2dbfaa89b57ec7b055afe29849ef7ddfeb1cefdb9ebdc43f5494984db29e5
        let base_point = s2pubkey("036d6caac248af96f6afa7f904f550253a0f3ef3f5aa2fe6838a95b216691468e2");
        let per_commitment_point = s2pubkey("025f7117a78150fe2ef97db7cfc83bd57b2e2c0d0dd25eaf467a4a1c2a45ce1486");
        let expected_pk = s2pubkey("0235f2dbfaa89b57ec7b055afe29849ef7ddfeb1cefdb9ebdc43f5494984db29e5");
        let pk = derive_pubkey(&base_point, &per_commitment_point);
        assert_eq!(&pk, &expected_pk);
    }

    #[test]
    fn test_derive_privkey() {
        // base_point_secret: 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
        // per_commitment_point: 025f7117a78150fe2ef97db7cfc83bd57b2e2c0d0dd25eaf467a4a1c2a45ce1486
        // resulting privkey: cbced912d3b21bf196a766651e436aff192362621ce317704ea2f75d87e7be0f
        let base_point_secret = s2privkey("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let per_commitment_point = s2pubkey("025f7117a78150fe2ef97db7cfc83bd57b2e2c0d0dd25eaf467a4a1c2a45ce1486");
        let expected_sk = s2privkey("cbced912d3b21bf196a766651e436aff192362621ce317704ea2f75d87e7be0f");
        let sk = derive_privkey(&base_point_secret, &per_commitment_point);
        assert_eq!(&sk, &expected_sk);
    }

    #[test]
    fn test_derive_revocation_pubkey() {
        // base_point 036d6caac248af96f6afa7f904f550253a0f3ef3f5aa2fe6838a95b216691468e2
        // per_commitment_point: 025f7117a78150fe2ef97db7cfc83bd57b2e2c0d0dd25eaf467a4a1c2a45ce1486
        // expected revocation key: 02916e326636d19c33f13e8c0c3a03dd157f332f3e99c317c141dd865eb01f8ff0
        let base_point = s2pubkey("036d6caac248af96f6afa7f904f550253a0f3ef3f5aa2fe6838a95b216691468e2");
        let per_commitment_point = s2pubkey("025f7117a78150fe2ef97db7cfc83bd57b2e2c0d0dd25eaf467a4a1c2a45ce1486");
        let expected_revocation_pk = s2pubkey("02916e326636d19c33f13e8c0c3a03dd157f332f3e99c317c141dd865eb01f8ff0");
        let revocation_pk = derive_revocation_pubkey(&base_point, &per_commitment_point);
        assert_eq!(revocation_pk, expected_revocation_pk);
    }

    #[test]
    fn test_derive_revocation_privkey() {
        // base_secret: 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
        // per_commitment_secret: 1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100
        // expected revocation secret: d09ffff62ddb2297ab000cc85bcb4283fdeb6aa052affbc9dddcf33b61078110
        let base_point_secret = s2privkey("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let per_commitment_point_secret = s2privkey("1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100");
        let expected_revocation_sk = s2privkey("d09ffff62ddb2297ab000cc85bcb4283fdeb6aa052affbc9dddcf33b61078110");
        let revocation_sk = derive_revocation_privkey(&base_point_secret, &per_commitment_point_secret);
        assert_eq!(revocation_sk, expected_revocation_sk);
    }
}