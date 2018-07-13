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

pub trait Functor {
    type Wrapped;

    fn fmap<F>(self, f: F) -> Self where F: FnOnce(Self::Wrapped) -> Self::Wrapped;
}

impl Functor for FeatureBit {
    type Wrapped = u16;

    fn fmap<F>(self, f: F) -> Self where F: FnOnce(Self::Wrapped) -> Self::Wrapped {
        Self::from(f(Self::Wrapped::from(self)))
    }
}

impl FeatureBit {
    pub fn pair(&self) -> Self {
        self.clone().fmap(|x| x ^ 1)
    }

    pub fn is_required(&self) -> bool {
        u16::from(self.clone()) & 1 == 0
    }
}

#[cfg(test)]
mod test {
    use super::FeatureBit;
    use super::Functor;

    #[test]
    fn correct() {
        // TODO: randomize it
        let feature_bit = FeatureBit::GossipQueriesOptional;

        assert_eq!(feature_bit.clone().fmap(|x| x), feature_bit);
    }
}
