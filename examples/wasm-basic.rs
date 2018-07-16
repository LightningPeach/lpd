extern crate lpd;
extern crate wire;

use wire::messages::Init;
use wire::feature::RawFeatureVector;
use wire::feature::FeatureBit;
use wire::serde_facade::BinarySD;

extern "C" {
    fn log(x: std::os::raw::c_int);
}

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

    unsafe { log(data.len() as _) }
}
