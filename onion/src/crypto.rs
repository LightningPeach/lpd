use wire::Hash256;
use chacha::{ChaCha, KeyStream};
use std::ops::BitXorAssign;
use serde_derive::{Serialize, Deserialize};

pub enum KeyType {
    Rho,
    Mu,
}

impl KeyType {
    // `KEY_LEN` is the length of the keys used to generate cipher streams and
    // encrypt payloads. Since we use SHA256 to generate the keys, the
    // maximum length currently is 32 bytes.
    const KEY_LEN: usize = 32;

    fn key(&self, shared_key: Hash256) -> [u8; Self::KEY_LEN] {
        use sha2::Sha256;
        use hmac::{Hmac, Mac};
        use self::KeyType::*;

        let key_type = match self {
            &Rho => "rho",
            &Mu => "mu",
        };

        let mut mac = Hmac::<Sha256>::new_varkey(key_type.as_bytes()).unwrap();
        mac.input(shared_key.as_ref());
        let result = mac.result().code();
        let mut array = [0; Self::KEY_LEN];
        array.copy_from_slice(result.as_slice());
        array
    }

    pub fn chacha(&self, shared_key: Hash256) -> ChaCha {
        ChaCha::new_chacha20(&self.key(shared_key), &[0u8; 8])
    }

    pub fn hmac(&self, shared_key: Hash256, msg: &[&[u8]]) -> HmacData {
        use sha2::Sha256;
        use hmac::{Hmac, Mac};

        let key = self.key(shared_key);
        let mac = Hmac::<Sha256>::new_varkey(&key).unwrap();
        let mut mac = msg.iter().fold(mac, |mut mac, &x| {
            mac.input(x);
            mac
        });
        let result = mac.result().code();
        let mut hmac = HmacData::default();
        hmac.data.copy_from_slice(result.as_slice());
        hmac
    }
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HmacData {
    data: [u8; 32],
}

impl HmacData {
    pub const SIZE: usize = 32;

    pub fn is_zero(&self) -> bool {
        self.data == [0; 32]
    }
}

impl<'a> BitXorAssign<&'a mut ChaCha> for HmacData {
    fn bitxor_assign(&mut self, rhs: &'a mut ChaCha) {
        rhs.xor_read(&mut self.data[..]).unwrap()
    }
}
