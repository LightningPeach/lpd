use std::fmt::Debug;

#[derive(Debug)]
pub struct UseOnce<T> where T: Debug, {
    data: Option<T>
}

impl<T> From<T> for UseOnce<T> where T: Debug, {
    fn from(v: T) -> Self {
        UseOnce {
            data: Some(v),
        }
    }
}

impl<T> UseOnce<T> where T: Debug, {
    pub fn consume(&mut self) -> Option<T> {
        use std::mem;

        let mut temp = None;
        mem::swap(&mut temp, &mut self.data);
        if temp.is_none() {
            println!("{:?} should not be used twice, ignoring", self);
        }
        temp
    }
}
