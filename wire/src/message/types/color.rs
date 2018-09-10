use super::common::Module;

#[derive(Clone, Default, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Color {
    data: [u8; 3],
}

// basis constructors
pub trait ColorBasis {
    fn r() -> Self;
    fn g() -> Self;
    fn b() -> Self;
}

impl ColorBasis for Color {
    fn r() -> Self {
        Color { data: [0xff, 0x00, 0x00], }
    }

    fn g() -> Self {
        Color { data: [0x00, 0xff, 0x00], }
    }

    fn b() -> Self {
        Color { data: [0x00, 0x00, 0xff], }
    }
}

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

            Color::from_iter((0..3).map(|i| add_check(self.data[i], rhs.data[i])))
        }
    }

    impl Sub for Color {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            let sub_check = |a: u8, b: u8| {
                let temp = (a as i16) - (b as i16);
                if temp < 0 { 0u8 } else { temp as _ }
            };

            Color::from_iter((0..3).map(|i| sub_check(self.data[i], rhs.data[i])))
        }
    }

    impl Mul<f32> for Color {
        type Output = Self;

        fn mul(self, rhs: f32) -> Self::Output {
            let mul_check = |a: u8, b: f32| {
                let temp = (a as f32) * (b as f32);
                if temp < 0.0 { 0u8 } else { if temp > 255.0 { 255u8 } else { temp as _ } }
            };

            Color::from_iter((0..3).map(|i| mul_check(self.data[i], rhs)))
        }
    }

    impl Module<f32> for Color {
        fn dot(self, rhs: Self) -> f32 {
            (0..3).fold(0.0, |acc, i|
                acc + ((self.data[i] * rhs.data[i]) as f32) / (255.0 * 255.0)
            )
        }
    }
}
