extern crate lpd;
extern crate wire;

use wire::messages::Init;
use wire::feature::RawFeatureVector;
use wire::feature::FeatureBit;
use wire::serde_facade::BinarySD;

#[no_mangle]
pub extern "C" fn main() {
    use self::FeatureBit::*;

    let init = Init::new(
        RawFeatureVector::new()
            .set_bit(DataLossProtectOptional),
        RawFeatureVector::new()
            .set_bit(GossipQueriesOptional)
            .set_bit(DataLossProtectRequired),
    );

    let mut data = Vec::<u8>::new();
    BinarySD::serialize(&mut data, &init).unwrap();
    println!("{:?}", data);
}
