use crate::{
    erased_view::{ErasedView, ViewMarker},
    geom::Rectf,
    Interest, View, ViewId,
};

pub struct ViewNode<T: 'static> {
    pub view: ViewNodeSlot<T>,
    pub parent: Option<ViewId>,
    pub children: Vec<ViewId>, // TODO maybe use a small vec
    pub next: usize,

    pub rect: Rectf,
    pub interest: Interest,
}

impl<T: 'static> std::fmt::Debug for ViewNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewNode")
            .field("view", &self.view)
            .field("parent", &self.parent)
            .field("children", &self.children)
            .field("next", &self.next)
            .field("rect", &self.rect)
            .field("interest", &self.interest)
            .finish()
    }
}

impl<T: 'static> ViewNode<T> {
    pub const fn empty(parent: ViewId) -> Self {
        Self {
            parent: Some(parent),
            view: ViewNodeSlot::Vacant,
            children: Vec::new(),
            next: 0,
            interest: Interest::NONE,
            rect: Rectf::ZERO,
        }
    }

    pub fn occupied(view: impl View<T> + 'static) -> Self {
        Self {
            view: ViewNodeSlot::occupied(view),
            parent: None,
            children: Vec::new(),
            next: 0,
            interest: Interest::NONE,
            rect: Rectf::ZERO,
        }
    }
}

pub enum ViewNodeSlot<T: 'static> {
    Vacant,
    Occupied(Box<dyn ErasedView<State = T>>),
}

impl<T: 'static> std::fmt::Debug for ViewNodeSlot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vacant => write!(f, "Vacant"),
            Self::Occupied(args) => write!(f, "{:?}", &args),
        }
    }
}

impl<T: 'static> std::ops::Deref for ViewNodeSlot<T> {
    type Target = Box<dyn ErasedView<State = T>>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        let Self::Occupied(view) = self else {
            unreachable!()
        };
        view
    }
}

impl<T: 'static> std::ops::DerefMut for ViewNodeSlot<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self::Occupied(view) = self else {
            unreachable!()
        };
        view
    }
}

impl<T: 'static> Default for ViewNodeSlot<T> {
    fn default() -> Self {
        Self::Vacant
    }
}

impl<T: 'static> ViewNodeSlot<T> {
    pub fn take(&mut self) -> Option<Box<dyn ErasedView<State = T>>> {
        match std::mem::take(self) {
            Self::Vacant => None,
            Self::Occupied(erased_view) => Some(erased_view),
        }
    }

    pub fn inhabit(&mut self, view: Box<dyn ErasedView<State = T>>) {
        assert!(matches!(self, Self::Vacant { .. }));
        *self = Self::Occupied(view)
    }

    fn occupied(view: impl View<T> + 'static) -> Self {
        Self::Occupied(Box::new(ViewMarker::new(view)))
    }
}
