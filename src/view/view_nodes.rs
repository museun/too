use std::{any::TypeId, collections::VecDeque};

use slotmap::SlotMap;

use super::{internal_views::Root, Erased, Ui, View, ViewId};
use crate::lock::{Lock, Ref, RefMapped, RefMut, RefMutMapped};

/// The persistent tree of all of the views.
pub struct ViewNodes {
    nodes: Lock<SlotMap<ViewId, ViewNode>>,
    stack: Lock<Vec<ViewId>>,
    removed: Lock<Vec<ViewId>>,
    pub(super) root: ViewId,
}

impl std::fmt::Debug for ViewNodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewNodes")
            .field("root", &self.root)
            .finish()
    }
}

impl ViewNodes {
    pub(super) fn new() -> Self {
        let mut nodes = SlotMap::with_key();
        let root = nodes.insert(ViewNode {
            view: Lock::new(Slot::new(Root)),
            ..ViewNode::default()
        });

        Self {
            nodes: Lock::new(nodes),
            stack: Lock::default(),
            removed: Lock::default(),
            root,
        }
    }

    pub(super) fn start(&mut self) {
        let root = self.root;
        self.nodes.get_mut()[root].next = 0;
    }

    pub(super) fn finish(&mut self) -> impl ExactSizeIterator<Item = ViewId> + use<'_> {
        self.removed.get_mut().drain(..)
    }

    pub(super) fn begin(&self, id: ViewId) {
        self.stack.borrow_mut().push(id);
    }

    pub(super) fn end(&self, id: ViewId) {
        let Some(old) = self.stack.borrow_mut().pop() else {
            unreachable!("stack was empty");
        };
        assert_eq!(old, id, "begin id: {id:?} did not match end id: {old:?}")
    }

    #[track_caller]
    pub(in crate::view) fn begin_view<V>(&self, args: V::Args<'_>, ui: &Ui) -> (ViewId, V::Response)
    where
        V: View,
    {
        let parent = self.current();
        let (id, resp) = self.update_view::<V>(parent, args, ui);
        (id, resp)
    }

    fn update_view<V>(&self, parent: ViewId, args: V::Args<'_>, ui: &Ui) -> (ViewId, V::Response)
    where
        V: View,
    {
        let Some(id) = self.append_view(parent) else {
            let (id, resp) = self.allocate_view::<V>(parent, args);
            self.stack.borrow_mut().push(id);
            return (id, resp);
        };

        let type_id = self.nodes.borrow()[id].view.borrow().type_id();
        if type_id != TypeId::of::<V>() {
            self.remove_view(id);
            let (id, resp) = self.allocate_view::<V>(parent, args);
            self.stack.borrow_mut().push(id);
            return (id, resp);
        }

        self.stack.borrow_mut().push(id);
        self.nodes.borrow_mut()[id].next = 0;

        let resp = self
            .scoped(id, |node| {
                let Some(view) = node.as_mut_any().downcast_mut::<V>() else {
                    unreachable!(
                        "type did not match: {} | {}",
                        node.type_name(),
                        std::any::type_name::<V>()
                    );
                };
                view.update(args, ui)
            })
            .unwrap();

        (id, resp)
    }

    fn append_view(&self, parent: ViewId) -> Option<ViewId> {
        let parent = &mut self.nodes.borrow_mut()[parent];
        let id = parent.children.get(parent.next).copied()?;
        parent.next += 1;
        Some(id)
    }

    fn allocate_view<V: View>(
        &self,
        parent_id: ViewId,
        args: V::Args<'_>,
    ) -> (ViewId, V::Response) {
        let view = V::create(args);

        let id = self.nodes.borrow_mut().insert(ViewNode {
            parent: Some(parent_id),
            view: Lock::new(Slot::new(view)),
            ..ViewNode::default()
        });

        let parent = &mut self.nodes.borrow_mut()[parent_id];
        if parent.next < parent.children.len() {
            parent.children[parent.next] = id;
        } else {
            parent.children.push(id);
        }
        parent.next += 1;

        (id, V::Response::default())
    }

    fn remove_view(&self, root: ViewId) {
        let mut queue = VecDeque::from([root]);

        let mut nodes = self.nodes.borrow_mut();
        let mut removed = self.removed.borrow_mut();

        while let Some(id) = queue.pop_front() {
            removed.push(id);

            let Some(node) = nodes.remove(id) else {
                continue;
            };

            queue.extend(&node.children);
            let Some(parent) = node.parent else {
                continue;
            };
            let Some(parent) = nodes.get_mut(parent) else {
                continue;
            };
            let len = parent.children.len();
            parent.children.retain(|&child| child != id);

            let difference = len.abs_diff(parent.children.len());
            parent.next = parent.next.saturating_sub(difference);
        }
    }

    pub(in crate::view) fn end_view(&self, id: ViewId) {
        let Some(old) = self.stack.borrow_mut().pop() else {
            unreachable!("called end view without an active view")
        };
        assert_eq!(
            id, old,
            "end view ({id:?}) did not much begin view ({old:?})"
        );
        self.cleanup(id);
    }

    fn cleanup(&self, start: ViewId) {
        // FIXME NLL 2024
        {
            let nodes = self.nodes.borrow();
            let node = &nodes[start];
            if node.next >= node.children.len() {
                return;
            }
        }

        let mut nodes = self.nodes.borrow_mut();
        let node = &mut nodes[start];

        let children = &node.children[node.next..];
        let mut queue = VecDeque::from_iter(children.iter().copied());
        node.children.truncate(node.next);

        let mut removed = self.removed.borrow_mut();

        while let Some(id) = queue.pop_front() {
            removed.push(id);
            let Some(next) = nodes.remove(id) else {
                unreachable!("child {id:?} should exist for {start:?}");
            };
            queue.extend(&next.children);
        }
    }

    /// Get the rood id for this tree
    pub const fn root(&self) -> ViewId {
        self.root
    }

    /// Tries to get a view by its id
    pub fn get(&self, id: ViewId) -> Option<RefMapped<'_, ViewNode>> {
        let nodes = self.nodes.borrow();
        Ref::filter_map(nodes, |nodes| nodes.get(id))
    }

    /// Tries to get a view by its id, mutably
    pub fn get_mut(&self, id: ViewId) -> Option<RefMutMapped<'_, ViewNode>> {
        let nodes = self.nodes.borrow_mut();
        RefMut::filter_map(nodes, |nodes| nodes.get_mut(id))
    }

    // TODO this should push the id to the stack and pop it off
    // TODO this should handle views not in the layout
    pub(super) fn scoped<R>(
        &self,
        id: ViewId,
        act: impl FnOnce(&mut dyn Erased) -> R,
    ) -> Option<R> {
        let nodes = self.nodes.borrow();
        let node = nodes.get(id)?;
        let mut view = node.view.borrow_mut().take();
        drop(nodes); // drop the borrow so we can recursively call this function

        let resp = act(&mut *view);

        let nodes = self.nodes.borrow();
        let node = nodes.get(id)?;
        node.view.borrow_mut().give(view);
        Some(resp)
    }

    /// Get the id of the current view
    pub fn current(&self) -> ViewId {
        self.stack.borrow().last().copied().unwrap_or(self.root)
    }

    /// Get the id of the current view's parent
    pub fn parent(&self) -> ViewId {
        self.stack
            .borrow()
            .iter()
            .nth_back(1)
            .copied()
            .unwrap_or(self.root)
    }

    /// Get the current [`ViewNode`]
    pub fn get_current(&self) -> RefMapped<'_, ViewNode> {
        let index = self.current();
        let nodes = self.nodes.borrow();
        Ref::map(nodes, |nodes| &nodes[index])
    }
}

/// A node for a view into the main UI tree
#[derive(Default)]
pub struct ViewNode {
    /// Your parents id, if you have one.
    pub parent: Option<ViewId>,
    /// Your childrens ids, if you have any
    pub children: Vec<ViewId>,
    pub(in crate::view) view: Lock<Slot>,
    pub(in crate::view) next: usize,
}

impl std::fmt::Debug for ViewNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewNode")
            .field("parent", &self.parent)
            .field("children", &self.children)
            .field("next", &self.next)
            .finish()
    }
}

// this is the only thing that has to be maybe send+sync
#[derive(Default)]
pub(in crate::view) enum Slot {
    #[default]
    Vacant,
    Inhabited(Box<dyn Erased>),
}

impl std::fmt::Debug for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vacant => write!(f, "Vacant"),
            Self::Inhabited(view) => view.fmt(f),
        }
    }
}

impl Slot {
    pub fn new(view: impl View + 'static) -> Self {
        Self::Inhabited(Box::new(view))
    }

    pub fn give(&mut self, view: Box<dyn Erased>) {
        assert!(matches!(self, Self::Vacant));
        *self = Self::Inhabited(view)
    }

    pub fn take(&mut self) -> Box<dyn Erased> {
        let Self::Inhabited(view) = std::mem::take(self) else {
            unreachable!("slot was vacant")
        };
        view
    }
}

impl std::ops::Deref for Slot {
    type Target = Box<dyn Erased>;
    #[inline(always)]
    #[track_caller]
    fn deref(&self) -> &Self::Target {
        let Self::Inhabited(view) = self else {
            unreachable!("slot was vacant")
        };
        view
    }
}

impl std::ops::DerefMut for Slot {
    #[inline(always)]
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self::Inhabited(view) = self else {
            unreachable!("slot was vacant")
        };
        view
    }
}
