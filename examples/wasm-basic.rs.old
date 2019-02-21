extern crate lpd;
extern crate wire;

use wire::*;

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
            .set_bit(DataLossProtectRequired)
            .set_bit(Custom(100500)),
    );

    let mut data = Vec::<u8>::new();
    BinarySD::serialize(&mut data, &init).unwrap();
    println!("{:?}", data);

    unsafe { log(data.len() as _) }
}
