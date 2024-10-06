use crate::view::{geom, Builder, View};

#[derive(Copy, Clone, Debug, PartialEq)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Flex {
    flex: geom::Flex,
}

impl Flex {
    pub fn new(flex: impl Into<geom::Flex>) -> Self {
        Self { flex: flex.into() }
    }
}

impl<'v> Builder<'v> for Flex {
    type View = Self;
}

impl View for Flex {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn flex(&self) -> geom::Flex {
        self.flex
    }
}
