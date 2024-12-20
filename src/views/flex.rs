use crate::{
    layout::Flex,
    view::{Builder, View},
};

#[derive(Copy, Clone, PartialEq)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Flexible {
    flex: Flex,
}

impl std::fmt::Debug for Flexible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.flex.fmt(f)
    }
}

impl Flexible {
    pub fn new(flex: impl Into<Flex>) -> Self {
        Self { flex: flex.into() }
    }
}

impl<'v> Builder<'v> for Flexible {
    type View = Self;
}

impl View for Flexible {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn flex(&self) -> Flex {
        self.flex
    }
}
