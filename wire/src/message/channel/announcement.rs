use super::RawFeatureVector;
use super::Hash256;
use super::PublicKey;
use super::ShortChannelId;
use super::Signed;
use super::SignedData;

pub type AnnouncementChannel = Signed<Signed<Signed<Signed<SignedData<AnnouncementChannelData>>>>>;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct AnnouncementChannelData {
    features: RawFeatureVector,
    chain_hash: Hash256,
    short_channel_id: ShortChannelId,
    pub node_id: (PublicKey, PublicKey),
    pub bitcoin_key: (PublicKey, PublicKey),
}

impl AnnouncementChannelData {
    pub fn hash(&self) -> &Hash256 {
        &self.chain_hash
    }

    pub fn id(&self) -> &ShortChannelId {
        &self.short_channel_id
    }
}

impl AnnouncementChannel {
    pub fn check_signatures(self) -> Result<AnnouncementChannelData, ()> {
        Ok(self
            .verify_owned(|data| &data.node_id.0).map_err(|_| ())?
            .verify_owned(|data| &data.node_id.1).map_err(|_| ())?
            .verify_owned(|data| &data.bitcoin_key.0).map_err(|_| ())?
            .verify_owned(|data| &data.bitcoin_key.1).map_err(|_| ())?
            .0)
    }

    pub fn check_features(&self, this: &RawFeatureVector) -> Result<(), ()> {
        // TODO: check_features
        let _ = this;
        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::*;
    use super::super::DataToSign;
    use ::BinarySD;

    #[test]
    fn announcement_channel() {
        let v = vec! [
            169u8, 177, 196, 25, 57, 80, 208, 176, 113, 192, 129, 194, 129, 60, 75, 12,
            21, 77, 188, 167, 162, 88, 249, 147, 231, 18, 208, 195, 174, 189, 240, 95,
            66, 108, 150, 147, 28, 77, 128, 69, 220, 78, 55, 45, 9, 120, 107, 254,
            154, 144, 165, 228, 138, 174, 67, 16, 90, 251, 148, 174, 188, 40, 216, 163,

            67, 115, 33, 54, 65, 131, 154, 187, 92, 226, 78, 198, 212, 93, 223, 21,
            144, 23, 40, 58, 253, 210, 118, 240, 234, 246, 211, 83, 4, 42, 57, 55,
            44, 231, 165, 215, 225, 114, 189, 99, 152, 241, 28, 69, 98, 36, 77, 240,
            114, 117, 137, 137, 43, 40, 197, 122, 204, 118, 250, 86, 53, 126, 9, 154,

            227, 178, 2, 243, 149, 135, 164, 247, 119, 8, 47, 214, 101, 138, 142, 71,
            238, 246, 115, 116, 111, 204, 23, 56, 137, 242, 32, 9, 193, 227, 7, 96,
            87, 154, 148, 14, 10, 143, 6, 44, 60, 186, 158, 171, 49, 31, 67, 18,
            69, 82, 223, 147, 47, 251, 152, 172, 55, 128, 80, 185, 36, 161, 114, 70,

            22, 193, 28, 214, 13, 181, 133, 248, 78, 134, 16, 44, 150, 133, 241, 129,
            82, 231, 247, 160, 106, 6, 231, 242, 125, 97, 79, 59, 94, 47, 201, 90,
            105, 171, 176, 101, 155, 38, 181, 222, 239, 138, 217, 90, 194, 85, 36, 49,
            125, 184, 112, 152, 123, 14, 232, 246, 241, 126, 176, 138, 200, 5, 243, 63,

            0, 0,

            246, 122, 215, 105, 93, 155, 102, 42, 114, 255, 61, 142, 219, 187, 45, 224,
            191, 166, 123, 19, 151, 75, 185, 145, 13, 17, 109, 92, 189, 134, 62, 104,

            0, 1, 145, 0, 0, 1, 0, 0,

            2, 248, 43, 81, 169, 251, 145, 163, 38, 87, 140, 176, 226, 78, 83, 136, 4, 246, 201, 235, 41, 126, 214, 0, 138, 132, 211, 64, 135, 97, 227, 175, 200,
            3, 138, 59, 70, 133, 145, 48, 34, 87, 182, 67, 158, 181, 248, 107, 90, 90, 147, 24, 111, 103, 186, 235, 35, 222, 132, 178, 111, 201, 198, 152, 199, 181,
            2, 68, 105, 45, 3, 43, 50, 104, 202, 38, 212, 250, 56, 173, 171, 55, 92, 149, 152, 44, 32, 44, 81, 36, 216, 168, 154, 73, 142, 101, 247, 192, 48,
            2, 44, 199, 59, 73, 153, 4, 138, 110, 45, 6, 200, 74, 184, 2, 205, 187, 124, 135, 83, 223, 253, 42, 27, 173, 32, 91, 76, 212, 219, 161, 117, 40,
        ];

        let t: AnnouncementChannel = BinarySD::deserialize(&v[..]).unwrap();
        let t = t
            .verify_owned(|data| &data.node_id.0).ok().unwrap()
            .verify_owned(|data| &data.node_id.1).ok().unwrap()
            .verify_owned(|data| &data.bitcoin_key.0).ok().unwrap()
            .verify_owned(|data| &data.bitcoin_key.1).ok().unwrap();

        println!("{:?}", t.as_ref_data());
    }
}
