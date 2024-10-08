use super::{
    erased_view::{ErasedView, ViewMarker},
    geom::Rectf,
    Interest, View, ViewId,
};

pub struct ViewNode<T: 'static> {
    pub view: NodeSlot<Box<dyn ErasedView<State = T>>>,
    pub parent: Option<ViewId>,
    pub children: Vec<ViewId>, // TODO maybe use a small vec
    pub next: usize,

    pub rect: Rectf,
    pub interest: Interest,
}

impl<T: 'static> ViewNode<T> {
    pub const fn empty(parent: ViewId) -> Self {
        Self {
            parent: Some(parent),
            view: NodeSlot::Vacant,
            children: Vec::new(),
            next: 0,
            interest: Interest::NONE,
            rect: Rectf::ZERO,
        }
    }

    pub fn occupied(view: impl View<T> + 'static) -> Self {
        Self {
            view: NodeSlot::occupied(view),
            parent: None,
            children: Vec::new(),
            next: 0,
            interest: Interest::NONE,
            rect: Rectf::ZERO,
        }
    }
}

pub(crate) enum NodeSlot<T: 'static> {
    Vacant,
    Occupied(T),
}

impl<T: 'static + std::fmt::Debug> std::fmt::Debug for NodeSlot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vacant => write!(f, "Vacant"),
            Self::Occupied(args) => write!(f, "{:?}", &args),
        }
    }
}

impl<T: 'static> std::ops::Deref for NodeSlot<T> {
    type Target = T;
    #[inline(always)]
    #[track_caller]
    fn deref(&self) -> &Self::Target {
        let Self::Occupied(view) = self else {
            unreachable!()
        };
        view
    }
}

impl<T: 'static> std::ops::DerefMut for NodeSlot<T> {
    #[inline(always)]
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self::Occupied(view) = self else {
            unreachable!()
        };
        view
    }
}

impl<T: 'static> Default for NodeSlot<T> {
    fn default() -> Self {
        Self::Vacant
    }
}

impl<T: 'static> NodeSlot<T> {
    pub fn take(&mut self) -> Option<T> {
        match std::mem::take(self) {
            Self::Vacant => None,
            Self::Occupied(node) => Some(node),
        }
    }

    pub const fn is_occupied(&self) -> bool {
        matches!(self, Self::Occupied(..))
    }

    #[track_caller]
    pub fn inhabit(&mut self, node: T) {
        assert!(matches!(self, Self::Vacant { .. }));
        *self = Self::Occupied(node)
    }

    pub fn as_ref(&self) -> &T {
        self
    }

    pub fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: 'static> NodeSlot<Box<dyn ErasedView<State = T>>> {
    fn occupied(view: impl View<T> + 'static) -> Self {
        Self::Occupied(Box::new(ViewMarker::new(view)))
    }
}
