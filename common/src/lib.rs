
pub trait Functor {
    type Wrapped;

    fn fmap<F>(self, f: F) -> Self where F: FnOnce(Self::Wrapped) -> Self::Wrapped;
}
