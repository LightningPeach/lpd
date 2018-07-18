use super::Wrapper;

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct Satoshi {
    raw: u64,
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct MilliSatoshi {
    raw: u64,
}

const MILE: u64 = 1000;

impl From<Satoshi> for MilliSatoshi {
    fn from(v: Satoshi) -> Self {
        MilliSatoshi {
            raw: v.raw * MILE,
        }
    }
}

impl From<MilliSatoshi> for Satoshi {
    fn from(v: MilliSatoshi) -> Self {
        Satoshi {
            raw: v.raw / MILE,
        }
    }
}

// TODO: write custom derive for `Wrapper` and `BiWrapper`
impl Wrapper for Satoshi {
    type Wrapped = u64;

    fn fmap<F>(self, f: F) -> Self where F: FnOnce(Self::Wrapped) -> Self::Wrapped {
        let Satoshi { raw: raw } = self;
        Satoshi { raw: f(raw) }
    }
}

impl Wrapper for MilliSatoshi {
    type Wrapped = u64;

    fn fmap<F>(self, f: F) -> Self where F: FnOnce(Self::Wrapped) -> Self::Wrapped {
        let MilliSatoshi { raw: raw } = self;
        MilliSatoshi { raw: f(raw) }
    }
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct SatoshiPerKiloWeight {
    raw: u32,
}

impl Wrapper for SatoshiPerKiloWeight {
    type Wrapped = u32;

    fn fmap<F>(self, f: F) -> Self where F: FnOnce(Self::Wrapped) -> Self::Wrapped {
        let SatoshiPerKiloWeight { raw: raw } = self;
        SatoshiPerKiloWeight { raw: f(raw) }
    }
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct CsvDelay {
    raw: u16,
}

impl Wrapper for CsvDelay {
    type Wrapped = u16;

    fn fmap<F>(self, f: F) -> Self where F: FnOnce(Self::Wrapped) -> Self::Wrapped {
        let CsvDelay { raw: raw } = self;
        CsvDelay { raw: f(raw) }
    }
}
