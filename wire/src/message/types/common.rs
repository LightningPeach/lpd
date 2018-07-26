/// Actually it is a Functor analogue,
/// but it is not clear (for me) if this is equivalent
/// to the true functor considering rust's semantic.
pub trait Wrapper {
    type Wrapped;

    fn fmap<F>(self, f: F) -> Self where F: FnOnce(Self::Wrapped) -> Self::Wrapped;
}

pub trait BiWrapper: Sized {
    type WrappedLeft;
    type WrappedRight;

    fn bimap<F, G>(self, f: F, g: G) -> Self where
        F: FnOnce(Self::WrappedLeft) -> Self::WrappedLeft,
        G: FnOnce(Self::WrappedRight) -> Self::WrappedRight
    {
        self.first(f).second(g)
    }

    fn first<F>(self, f: F) -> Self where F: FnOnce(Self::WrappedLeft) -> Self::WrappedLeft;
    fn second<G>(self, g: G) -> Self where G: FnOnce(Self::WrappedRight) -> Self::WrappedRight;
}
