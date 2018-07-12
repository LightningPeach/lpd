use std::collections::HashMap;

use super::RawFeatureVector;
use super::FeatureBit;

pub struct FeatureVector {
    raw: RawFeatureVector,
    names: HashMap<FeatureBit, String>,
}
