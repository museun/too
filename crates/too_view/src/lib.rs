#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
use std::{any::TypeId, collections::VecDeque};

use slotmap::{SecondaryMap, SlotMap};
use too_renderer::SurfaceMut;

pub mod geom;
use geom::{Point, Rectf, Size, Space, Vector};

pub mod views;

mod response;
pub use response::Response;

mod erased_view;
use erased_view::{ErasedView, ViewMarker};

pub mod view;
use too_runner::layout::{Anchor2, LinearLayout};
use too_shapes::Text;
use view::Context;
pub use view::{Args, NoArgs, NoResponse, View, ViewExt};

mod view_node;
use view_node::ViewNode;

mod app;
pub use app::{App, AppRunner};

slotmap::new_key_type! {
    pub struct ViewId;
}

mod input;
use input::Input;
pub use input::{Event, EventCtx, Handled, Interest};

#[derive(Debug)]
struct LayoutNode {
    rect: Rectf,
    interest: Interest,
}

pub struct LayoutCtx<'a, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub state: &'a mut T,

    client_rect: Rectf,
    input: &'a mut Input,
    nodes: &'a mut SlotMap<ViewId, Option<ViewNode<T>>>,
    computed: &'a mut SecondaryMap<ViewId, LayoutNode>,
    stack: &'a mut Vec<ViewId>,
    debug: &'a mut Vec<String>,
}

impl<'a, T: 'static> LayoutCtx<'a, T> {
    pub fn compute_layout(&mut self, child: ViewId, space: Space) -> Size {
        let Some(node) = self.nodes.get_mut(child) else {
            return Size::ZERO;
        };

        let Some(mut node) = node.take() else {
            unreachable!("node: {child:?} was missing")
        };

        self.stack.push(child);

        let size = node.view.layout(
            LayoutCtx {
                current_id: child,
                children: &node.children,
                client_rect: self.client_rect,
                state: self.state,
                input: self.input,
                nodes: self.nodes,
                computed: self.computed,
                stack: self.stack,
                debug: self.debug,
            },
            space,
        );

        let is_new_mouse_layer = self.input.mouse.current_layer_root() == Some(child);
        let is_new_keyboard_layer = self.input.keyboard.current_layer_root() == Some(child);

        let interest = node.view.interest();
        if interest.is_mouse_any() {
            self.input.mouse.add(child, interest);
        }
        if interest.is_key_input() {
            self.input.keyboard.add(child);
        }

        if is_new_mouse_layer {
            self.input.mouse.pop_layer();
        }
        if is_new_keyboard_layer {
            self.input.keyboard.pop_layer();
        }

        // is it here? (center { widget { another widget }})
        let rect = Rectf::from(size.clamp(Size::ZERO, self.client_rect.size()));
        self.computed.insert(child, LayoutNode { rect, interest });

        assert_eq!(Some(child), self.stack.pop());
        assert!(self.nodes[child].replace(node).is_none());

        size
    }

    pub fn new_layer_for(&mut self, id: ViewId) {
        self.input.mouse.push_layer(id);
        self.input.keyboard.push_layer(id);
    }

    pub fn new_layer(&mut self) {
        self.new_layer_for(self.current_id);
    }

    pub fn set_position(&mut self, child: ViewId, offset: impl Into<Vector>) {
        if let Some(node) = self.computed.get_mut(child) {
            node.rect += offset.into();
        }
    }

    pub fn set_size(&mut self, child: ViewId, size: impl Into<Size>) {
        if let Some(node) = self.computed.get_mut(child) {
            node.rect += size.into()
        }
    }

    pub fn debug(&mut self, msg: impl ToString) {
        self.debug.push(msg.to_string());
    }
}

pub struct DrawCtx<'a, 'c: 't, 't, T: 'static> {
    pub rect: Rectf,
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub surface: &'t mut SurfaceMut<'c>,
    pub state: &'a mut T,
    nodes: &'a mut SlotMap<ViewId, Option<ViewNode<T>>>,
    computed: &'a SecondaryMap<ViewId, LayoutNode>,
    stack: &'a mut Vec<ViewId>,
    debug: &'a mut Vec<String>,
}

impl<'a, 'c: 't, 't, T: 'static> DrawCtx<'a, 'c, 't, T> {
    pub fn draw(&mut self, id: ViewId) {
        let Some(layout) = self.computed.get(id) else {
            return;
        };

        self.stack.push(id);

        let Some(node) = self.nodes.get_mut(id) else {
            unreachable!("root node should always exist")
        };

        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", id)
        };

        let ctx = DrawCtx {
            rect: layout.rect,
            current_id: id,
            children: &node.children,
            surface: &mut self.surface.crop(layout.rect.into()),
            state: self.state,
            nodes: self.nodes,
            computed: self.computed,
            stack: self.stack,
            debug: self.debug,
        };

        node.view.draw(ctx);
        assert_eq!(Some(id), self.stack.pop());
        assert!(self.nodes[id].replace(node).is_none());
    }

    pub fn debug(&mut self, msg: impl ToString) {
        self.debug.push(msg.to_string());
    }
}

// TODO somehow integrate with the `Context` from too_runner (to send commands to the backend)
pub struct Ui<T: 'static> {
    // TODO we just need an arena for this
    // we can store the layout rect on the node
    //
    // Option so we can do a take/insert dance
    // FIXME this is highly annoying
    nodes: SlotMap<ViewId, Option<ViewNode<T>>>,
    // if we do keep the slot map, this can just be a hashmap
    computed: SecondaryMap<ViewId, LayoutNode>,
    root: ViewId,

    input: Input,

    stack: Vec<ViewId>,
    removed: Vec<ViewId>,
    // TODO reuse vedeque from the BFS
    rect: Rectf,
    quit: bool,

    debug: Vec<String>,
}

impl<T: 'static> std::fmt::Debug for Ui<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ui")
            .field(
                "type",
                &crate::debug_fmt::str(&debug_fmt::short_name(std::any::type_name::<T>())),
            )
            .field("nodes", &crate::debug_fmt::slot_map(&self.nodes))
            .field("computed", &crate::debug_fmt::secondary_map(&self.computed))
            .field("root", &crate::debug_fmt::id(self.root))
            .field("input", &self.input)
            .finish()
    }
}

impl<T: 'static> Ui<T> {
    pub fn request_quit(&mut self) {
        self.quit = true
    }

    pub fn root(&self) -> ViewId {
        self.root
    }

    pub fn current(&self) -> ViewId {
        self.stack.last().copied().unwrap_or(self.root())
    }

    // TODO figure out a better way for this
    pub fn ctx<'a>(&'a mut self, state: &'a mut T) -> view::Context<'a, T> {
        view::Context { ui: self, state }
    }
}

impl<T: 'static> Ui<T> {
    fn new(rect: impl Into<Rectf>) -> Self {
        let mut nodes = SlotMap::with_key();
        Self {
            root: nodes.insert(Some(ViewNode::occupied(views::RootView))),
            nodes,
            computed: SecondaryMap::new(),

            stack: Vec::new(),
            removed: Vec::new(),

            input: Input::default(),

            rect: rect.into(),
            quit: false,

            debug: Vec::new(),
        }
    }

    fn scope(&mut self, state: &mut T, apply: fn(&mut Context<'_, T>)) {
        self.begin();
        apply(&mut self.ctx(state));
        self.end(state);
    }

    fn begin(&mut self) {
        self.nodes[self.root].as_mut().unwrap().next = 0;
        self.input.begin();
    }

    fn end(&mut self, state: &mut T) {
        self.removed.clear();
        self.cleanup(self.root);
        self.input.end(&self.removed);

        self.layout(state);
        self.resolve();
    }

    fn resolve(&mut self) {
        let Some(root) = &self.nodes[self.root] else {
            unreachable!("root node {:?} was not found", self.root);
        };

        let mut queue = VecDeque::from_iter(root.children.iter().map(|&id| (id, Point::ZERO)));

        while let Some((id, pos)) = queue.pop_front() {
            let Some(node) = self.computed.get_mut(id) else {
                continue;
            };

            let offset = pos.to_vector();
            node.rect += offset;
            let Some(next) = &self.nodes[id] else {
                unreachable!("node: {id:?} was missing")
            };

            queue.extend(next.children.iter().map(|&id| (id, node.rect.min)))
        }
    }

    fn tick(&mut self, dt: f32) {}

    fn event(&mut self, state: &mut T, event: too_events::Event) {
        if let too_events::Event::Resize(size) = event {
            self.rect = Rectf::min_size(Point::ZERO, size.into());
        }

        self.input.handle(
            &event, //
            &mut self.nodes,
            &mut self.computed,
            state,
            &mut self.debug,
        );
    }

    fn layout(&mut self, state: &mut T) {
        let Some(node) = self.nodes.get_mut(self.root) else {
            unreachable!("root node should always exist")
        };

        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", self.root)
        };

        let ctx = LayoutCtx::<T> {
            current_id: self.root,
            children: &node.children,
            state,
            input: &mut self.input,
            client_rect: self.rect,
            nodes: &mut self.nodes,
            computed: &mut self.computed,
            stack: &mut self.stack,
            debug: &mut self.debug,
        };

        let space = Space {
            min: Size::ZERO,
            max: self.rect.size(),
        };

        let _ = node.view.layout(ctx, space);
        assert!(self.nodes[self.root].replace(node).is_none());
    }

    fn render(&mut self, state: &mut T, mut surface: SurfaceMut<'_>) {
        let Some(node) = self.nodes.get_mut(self.root) else {
            unreachable!("root node should always exist")
        };

        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", self.root)
        };

        let ctx = DrawCtx::<T> {
            rect: surface.rect().into(),
            current_id: self.root,
            children: &node.children,
            surface: &mut surface,
            state,
            nodes: &mut self.nodes,
            computed: &self.computed,
            stack: &mut self.stack,
            debug: &mut self.debug,
        };
        node.view.draw(ctx);
        assert!(self.nodes[self.root].replace(node).is_none());

        let mut alloc = LinearLayout::vertical()
            .anchor(Anchor2::RIGHT_TOP)
            .wrap(true)
            .layout(surface.rect());

        for debug in self.debug.drain(..) {
            let text = Text::new(debug);
            if let Some(rect) = alloc.allocate(text.size()) {
                surface.crop(rect).draw(text);
            }
        }
    }

    fn begin_view<V>(&mut self, state: &mut T, args: V::Args<'_>) -> Response<V::Response>
    where
        V: View<T> + 'static,
    {
        let parent = self.current();
        let (id, mut view) = self.update_view::<V>(args.clone(), parent);
        self.stack.push(id);

        let Some(actual_view) = view.as_any_mut().downcast_mut::<V>() else {
            unreachable!(
                "expected to get view: {}, got {}",
                std::any::type_name::<V>(),
                view.type_name()
            )
        };

        let resp = actual_view.update(state, args);
        self.nodes[id].as_mut().unwrap().view.inhabit(view);
        Response::new(id, resp, ()) // TODO what should `Response::inner` be?
    }

    fn end_view(&mut self, id: ViewId) {
        let Some(old) = self.stack.pop() else {
            unreachable!("called end view without an active view")
        };
        assert_eq!(id, old, "end view did not match input view");

        self.cleanup(id);
    }

    fn append_view(&mut self, id: ViewId) -> Option<ViewId> {
        let parent = self.nodes[id].as_mut().unwrap();
        let id = parent.children.get(parent.next).copied()?;
        parent.next += 1;
        Some(id)
    }

    fn allocate_view<V>(
        &mut self,
        args: V::Args<'_>,
        parent: ViewId,
    ) -> (ViewId, Box<dyn ErasedView<State = T>>)
    where
        V: View<T> + 'static,
    {
        let id = self.nodes.insert(Some(ViewNode::empty(parent)));

        let parent = self.nodes[parent].as_mut().unwrap();
        if parent.next < parent.children.len() {
            parent.children[parent.next] = id;
        } else {
            parent.children.push(id);
        }
        parent.next += 1;

        (id, Box::new(ViewMarker::new(V::create(args))))
    }

    fn update_view<V>(
        &mut self,
        args: V::Args<'_>,
        parent: ViewId,
    ) -> (ViewId, Box<dyn ErasedView<State = T>>)
    where
        V: View<T> + 'static,
    {
        let Some(id) = self.append_view(parent) else {
            return self.allocate_view::<V>(args, parent);
        };

        let Some(node) = self.nodes.get_mut(id).and_then(<Option<_>>::as_mut) else {
            unreachable!("node {id:?} must exist")
        };

        let Some(view) = node.view.take() else {
            unreachable!("node {id:?} was not occupied")
        };

        if view.as_ref().type_id() != TypeId::of::<ViewMarker<T, V>>() {
            self.remove_view(id);
            return self.allocate_view::<V>(args, parent);
        }

        node.next = 0;
        (id, view)
    }

    fn remove_view(&mut self, id: ViewId) {
        let mut queue = VecDeque::from_iter([id]);
        while let Some(id) = queue.pop_front() {
            self.removed.push(id);
            if let Some(node) = self.nodes.remove(id).flatten() {
                queue.extend(node.children);
                if let Some(parent) = node.parent {
                    if let Some(parent) = self.nodes.get_mut(parent).and_then(|s| s.as_mut()) {
                        parent.children.retain(|&child| child != id);
                    }
                }
            }
        }
    }

    fn cleanup(&mut self, start: ViewId) {
        let node = self.nodes[start].as_mut().unwrap();
        if node.next >= node.children.len() {
            return;
        }

        let children = &node.children[node.next..];
        let mut queue = VecDeque::from_iter(children.iter().copied());
        self.removed.extend_from_slice(children);
        node.children.truncate(node.next);

        while let Some(id) = queue.pop_front() {
            self.removed.push(id);
            let Some(next) = self.nodes.remove(id).flatten() else {
                unreachable!("child: {id:?} should exist for {start:?}")
            };
            queue.extend(next.children);
        }
    }
}

mod debug_fmt {
    use std::fmt::{Debug, Formatter, Result};

    use slotmap::{Key, SecondaryMap, SlotMap};

    use crate::ViewId;

    pub const fn str(s: &str) -> impl Debug + '_ {
        struct NoQuote<'a>(&'a str);
        impl<'a> Debug for NoQuote<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                f.write_str(self.0)
            }
        }
        NoQuote(s)
    }

    pub const fn opt_id(id: Option<ViewId>) -> impl Debug {
        struct ShortId(Option<ViewId>);
        impl Debug for ShortId {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                match self.0.map(|s| s.data()) {
                    Some(s) => write!(f, "{s:?}"),
                    None => write!(f, "None"),
                }
            }
        }
        ShortId(id)
    }

    pub const fn id(id: ViewId) -> impl Debug {
        struct ShortId(ViewId);
        impl Debug for ShortId {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                write!(f, "{:?}", self.0.data())
            }
        }
        ShortId(id)
    }

    pub const fn vec(list: &Vec<ViewId>) -> impl Debug + '_ {
        struct Inner<'a>(&'a Vec<ViewId>);
        impl<'a> Debug for Inner<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                f.debug_list()
                    .entries(self.0.iter().map(|&id| self::id(id)))
                    .finish()
            }
        }
        Inner(list)
    }

    pub const fn slot_map<T: Debug>(map: &SlotMap<ViewId, T>) -> impl Debug + '_ {
        struct Inner<'a, T>(&'a SlotMap<ViewId, T>);
        impl<'a, T: Debug> Debug for Inner<'a, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                f.debug_map()
                    .entries(self.0.iter().map(|(k, v)| (self::id(k), v)))
                    .finish()
            }
        }
        Inner(map)
    }

    pub fn secondary_map<T: Debug>(map: &SecondaryMap<ViewId, T>) -> impl Debug + '_ {
        struct Inner<'a, T>(&'a SecondaryMap<ViewId, T>);
        impl<'a, T: Debug> Debug for Inner<'a, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                f.debug_map()
                    .entries(self.0.iter().map(|(k, v)| (id(k), v)))
                    .finish()
            }
        }
        Inner(map)
    }

    pub fn short_name(name: &str) -> String {
        const fn is_special(c: char) -> bool {
            matches!(c, ' ' | '<' | '>' | '(' | ')' | '[' | ']' | ',' | ';')
        }

        fn collapse(s: &str) -> &str {
            s.split("::").last().unwrap()
        }

        let mut index = 0;
        let end = name.len();
        let mut out = String::new();

        while index < end {
            let rest = &name[index..end];
            if let Some(mut p) = rest.find(is_special) {
                out.push_str(collapse(&rest[0..p]));

                let ch = &rest[p..=p];
                out.push_str(ch);

                if matches!(ch, ">" | ")" | "]" if rest[p + 1..].starts_with("::")) {
                    out.push_str("::");
                    p += 2;
                }
                index += p + 1;
            } else {
                out.push_str(collapse(rest));
                index = end;
            }
        }
        out
    }
}
