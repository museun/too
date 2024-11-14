use super::{Builder, Response, Ui, View, ViewExt};

pub trait Adhoc<'v>: Sized {
    type Output: 'static;
    fn show(self, ui: &Ui) -> Self::Output;
}

impl<'v, T> Adhoc<'v> for T
where
    T: Builder<'v>,
{
    type Output = Response<<T::View as View>::Response>;
    fn show(self, ui: &Ui) -> Self::Output {
        <T as ViewExt>::show(self, ui)
    }
}
