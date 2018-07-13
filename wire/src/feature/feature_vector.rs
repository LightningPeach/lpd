use std::collections::HashMap;

use super::RawFeatureVector;
use super::FeatureBit;

pub struct FeatureVector {
    raw: RawFeatureVector,
    names: HashMap<FeatureBit, String>,
}

impl FeatureVector {
    pub fn new() -> Self {
        FeatureVector {
            raw: RawFeatureVector::new(),
            names: HashMap::new(),
        }
    }

    pub fn has(&self, feature_bit: &FeatureBit) -> bool {
        self.raw.is_set_bit(feature_bit) ||
            (self.is_pair(feature_bit) && self.raw.is_set_bit(&feature_bit.pair()))
    }

    pub fn name(&self, feature_bit: &FeatureBit) -> Option<String> {
        self.names.get(feature_bit).map(Clone::clone)
    }

    pub fn is_pair(&self, feature_bit: &FeatureBit) -> bool {
        let names = (
            self.name(feature_bit),
            self.name(&feature_bit.pair())
        );

        match names {
            (Some(_), Some(_)) => true,
            _ => false,
        }
    }
}
