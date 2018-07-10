#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub enum FeatureBit {
    DataLossProtectRequired,
    DataLossProtectOptional,
    InitialRoutingSync,
    GossipQueriesRequired,
    GossipQueriesOptional,
    Custom(u16),
}

impl From<u16> for FeatureBit {
    fn from(raw: u16) -> Self {
        use self::FeatureBit::*;
        match raw {
            0 => DataLossProtectRequired,
            1 => DataLossProtectOptional,
            3 => InitialRoutingSync,
            6 => GossipQueriesRequired,
            7 => GossipQueriesOptional,
            c @ _ => Custom(c),
        }
    }
}

impl From<FeatureBit> for u16 {
    fn from(feature_bit: FeatureBit) -> Self {
        use self::FeatureBit::*;
        match feature_bit {
            DataLossProtectRequired => 0,
            DataLossProtectOptional => 1,
            InitialRoutingSync => 3,
            GossipQueriesRequired => 6,
            GossipQueriesOptional => 7,
            Custom(c) => c,
        }
    }
}

impl FeatureBit {
    pub fn is_valid(&self) -> bool {
        FeatureBit::from(u16::from(self.clone())) == self.clone()
    }
}
