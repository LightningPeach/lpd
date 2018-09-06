use wire::SatoshiPerVByte;

pub trait FeeEstimator {
    fn estimate(&mut self, num_blocks: u32) -> SatoshiPerVByte;
}

#[allow(dead_code)]
pub struct StaticFeeEstimator {
    rate: SatoshiPerVByte,
}

impl StaticFeeEstimator {
    pub fn new(rate: SatoshiPerVByte) -> Self {
        StaticFeeEstimator {
            rate: rate,
        }
    }
}

impl FeeEstimator for StaticFeeEstimator {
    fn estimate(&mut self, num_block: u32) -> SatoshiPerVByte {
        let _ = num_block;
        self.rate
    }
}
