use dependencies::hex;

use super::common::Module;

// I guess it is RGBA
#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct Color {
    data: [u8; 4],
}

impl Color {
    pub fn from_u32(x: u32) -> Color {
        let mut data = [0u8; 4];
        data[0] = ((x >> 24) & 0xFF) as u8; // Red
        data[1] = ((x >> 16) & 0xFF) as u8; // Green
        data[2] = ((x >> 8) & 0xFF) as u8; // Blue
        data[3] = ((x >> 0) & 0xFF) as u8; // Alpha
        return Color {
            data
        }
    }
}

impl serde::Serialize for Color {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (self.data[0], self.data[1], self.data[2]).serialize(s)
    }
}

// TODO(mkl): maybe do not drop A value?
// TODO(mkl): maybe create type RGB color
impl<'de> serde::Deserialize<'de> for Color {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
        let v: (u8, u8, u8) = serde::Deserialize::deserialize(d)?;
        Ok(Color {
            data: [v.0, v.1, v.2, 0],
        })
    }
}

// TODO(mkl): maybe create enum Colors with different colors instead of adding functions to Color
// basis constructors
pub trait ColorBasis {
    fn r() -> Self;
    fn g() -> Self;
    fn b() -> Self;
    fn a() -> Self;
}

impl ColorBasis for Color {
    fn r() -> Self {
        Color { data: [0xff, 0x00, 0x00, 0x00], }
    }

    fn g() -> Self {
        Color { data: [0x00, 0xff, 0x00, 0x00], }
    }

    fn b() -> Self {
        Color { data: [0x00, 0x00, 0xff, 0x00], }
    }

    fn a() -> Self {
        Color { data: [0x00, 0x00, 0x00, 0xff], }
    }
}

mod debug {
    use super::Color;
    use super::hex::encode;
    use std::fmt;

    impl fmt::Display for Color {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", encode(self.data))
        }

    }
}

// TODO(mkl): add some tests
// TODO(mkl): add usability functions
mod module {
    use super::Color;
    use super::Module;

    use std::iter::FromIterator;
    use std::ops::Add;
    use std::ops::Sub;
    use std::ops::Mul;

    impl FromIterator<u8> for Color {
        fn from_iter<T: IntoIterator<Item=u8>>(iter: T) -> Self {
            let mut c = Color::default();
            c.data[0..].clone_from_slice(
                iter.into_iter()
                    .collect::<Vec<u8>>()
                    .as_slice()
            );

            c
        }
    }

    impl Add for Color {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            let add_check = |a: u8, b: u8| {
                let temp = (a as u16) + (b as u16);
                if temp > 255 { 255u8 } else { temp as _ }
            };

            Color::from_iter((0..4).map(|i| add_check(self.data[i], rhs.data[i])))
        }
    }

    impl Sub for Color {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            let sub_check = |a: u8, b: u8| {
                let temp = (a as i16) - (b as i16);
                if temp < 0 { 0u8 } else { temp as _ }
            };

            Color::from_iter((0..4).map(|i| sub_check(self.data[i], rhs.data[i])))
        }
    }

    impl Mul<f32> for Color {
        type Output = Self;

        fn mul(self, rhs: f32) -> Self::Output {
            let mul_check = |a: u8, b: f32| {
                let temp = (a as f32) * (b as f32);
                if temp < 0.0 { 0u8 } else { if temp > 255.0 { 255u8 } else { temp as _ } }
            };

            Color::from_iter((0..4).map(|i| mul_check(self.data[i], rhs)))
        }
    }

    impl Module<f32> for Color {
        fn dot(self, rhs: Self) -> f32 {
            (0..4).fold(0.0, |acc, i|
                acc + ((self.data[i] * rhs.data[i]) as f32) / (255.0 * 255.0)
            )
        }
    }
}
