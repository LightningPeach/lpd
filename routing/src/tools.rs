use std::fmt::Debug;

#[derive(Debug)]
pub struct GenericSystem<Input, Output>(Inner<Input, Output>)
where
    Input: Debug,
    Output: Debug;

#[derive(Debug)]
enum Inner<Input, Output>
where
    Input: Debug,
    Output: Debug,
{
    Ready(Input),
    // such option is always `Some`, so unwrap is safe
    Complete(Option<Output>),
}

impl<Input, Output> From<Input> for GenericSystem<Input, Output>
where
    Input: Debug,
    Output: Debug,
{
    fn from(v: Input) -> Self {
        GenericSystem(Inner::Ready(v))
    }
}

impl<Input, Output> GenericSystem<Input, Output>
where
    Input: Debug,
    Output: Debug,
{
    pub fn run_func<F>(&mut self, f: F) where F: FnOnce(Input) -> Output {
        use std::mem;

        let mut temp = GenericSystem(Inner::Complete(None));
        let mut s = self;
        mem::swap(&mut temp, &mut s);
        match temp {
            GenericSystem(Inner::Ready(input)) => {
                *s = GenericSystem(Inner::Complete(Some(f(input))));
            },
            GenericSystem(Inner::Complete(v)) => {
                println!("{:?} should not be used twice, ignoring", v.as_ref().unwrap());
                *s = GenericSystem(Inner::Complete(v));
            },
        }
    }

    pub fn output(self) -> Output {
        match self {
            GenericSystem(Inner::Ready(_)) => panic!("is not complete"),
            GenericSystem(Inner::Complete(t)) => t.unwrap(),
        }
    }
}
