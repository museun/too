#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
use std::{any::TypeId, collections::VecDeque};

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

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ViewId(thunderdome::Index);

mod input;
use input::Input;
pub use input::{Event, EventCtx, Handled, Interest};

pub struct LayoutCtx<'a, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub state: &'a mut T,

    client_rect: Rectf,
    input: &'a mut Input,
    nodes: &'a mut thunderdome::Arena<Option<ViewNode<T>>>,
    stack: &'a mut Vec<ViewId>,
    debug: &'a mut Vec<String>,
}

impl<'a, T: 'static> LayoutCtx<'a, T> {
    pub fn compute_layout(&mut self, child: ViewId, space: Space) -> Size {
        let Some(node) = self.nodes.get_mut(child.0) else {
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
        node.rect = rect;
        node.interest = interest;

        assert_eq!(Some(child), self.stack.pop());
        assert!(self.nodes[child.0].replace(node).is_none());

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
        if let Some(node) = self.nodes.get_mut(child.0) {
            let Some(node) = node else {
                unreachable!("node {child:?} is missing")
            };

            node.rect += offset.into();
        }
    }

    pub fn set_size(&mut self, child: ViewId, size: impl Into<Size>) {
        if let Some(node) = self.nodes.get_mut(child.0) {
            let Some(node) = node else {
                unreachable!("node {child:?} is missing")
            };
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

    nodes: &'a mut thunderdome::Arena<Option<ViewNode<T>>>,
    stack: &'a mut Vec<ViewId>,
    debug: &'a mut Vec<String>,
}

impl<'a, 'c: 't, 't, T: 'static> DrawCtx<'a, 'c, 't, T> {
    pub fn draw(&mut self, id: ViewId) {
        let Some(node) = self.nodes.get_mut(id.0) else {
            return;
        };

        // this is annoying but I think I solved it
        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", id)
        };

        self.stack.push(id);

        let ctx = DrawCtx {
            rect: node.rect,
            current_id: id,
            children: &node.children,
            surface: &mut self.surface.crop(node.rect.into()),
            state: self.state,
            nodes: self.nodes,
            stack: self.stack,
            debug: self.debug,
        };

        node.view.draw(ctx);
        assert_eq!(Some(id), self.stack.pop());
        assert!(self.nodes[id.0].replace(node).is_none());
    }

    pub fn debug(&mut self, msg: impl ToString) {
        self.debug.push(msg.to_string());
    }
}

pub struct Ui<T: 'static> {
    // Option so we can do a take/insert dance
    // FIXME this is highly annoying
    nodes: thunderdome::Arena<Option<ViewNode<T>>>,
    root: ViewId,

    input: Input,

    stack: Vec<ViewId>,
    removed: Vec<ViewId>,

    // TODO reuse vecdeque from the BFS
    rect: Rectf,
    quit: bool,

    debug: Vec<String>,
}

impl<T> std::fmt::Debug for Ui<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ui")
            .field("nodes", &self.nodes)
            .field("root", &self.root)
            .field("input", &self.input)
            .field("stack", &self.stack)
            .field("removed", &self.removed)
            .field("rect", &self.rect)
            .field("quit", &self.quit)
            .field("debug", &self.debug)
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

    pub fn ctx<'a>(&'a mut self, state: &'a mut T) -> view::Context<'a, T> {
        view::Context { ui: self, state }
    }
}

impl<T: 'static> Ui<T> {
    fn new(rect: impl Into<Rectf>) -> Self {
        let mut nodes = thunderdome::Arena::new();
        Self {
            root: ViewId(nodes.insert(Some(ViewNode::occupied(views::RootView)))),
            nodes,

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
        self.nodes[self.root.0].as_mut().unwrap().next = 0;
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
        let Some(root) = &self.nodes[self.root.0] else {
            unreachable!("root node {:?} was not found", self.root);
        };

        let mut queue = VecDeque::from_iter(root.children.iter().map(|&id| (id, Point::ZERO)));

        while let Some((id, pos)) = queue.pop_front() {
            let Some(node) = self.nodes.get_mut(id.0) else {
                continue;
            };

            let Some(next) = node.as_mut() else {
                unreachable!("node: {id:?} was missing")
            };

            let offset = pos.to_vector();
            next.rect += offset;

            queue.extend(next.children.iter().map(|&id| (id, next.rect.min)))
        }
    }

    fn tick(&mut self, dt: f32) {
        // TODO this needs to find the things that want animation
        // and do it
    }

    fn event(&mut self, state: &mut T, event: too_events::Event) {
        if let too_events::Event::Resize(size) = event {
            self.rect = Rectf::min_size(Point::ZERO, size.into());
        }

        self.input.handle(
            &event, //
            &mut self.nodes,
            state,
            &mut self.debug,
        );
    }

    fn layout(&mut self, state: &mut T) {
        let Some(node) = self.nodes.get_mut(self.root.0) else {
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
            stack: &mut self.stack,
            debug: &mut self.debug,
        };

        let space = Space {
            min: Size::ZERO,
            max: self.rect.size(),
        };

        let _ = node.view.layout(ctx, space);
        assert!(self.nodes[self.root.0].replace(node).is_none());
    }

    fn render(&mut self, state: &mut T, mut surface: SurfaceMut<'_>) {
        let Some(node) = self.nodes.get_mut(self.root.0) else {
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
            stack: &mut self.stack,
            debug: &mut self.debug,
        };
        node.view.draw(ctx);
        assert!(self.nodes[self.root.0].replace(node).is_none());

        // TODO this could be done with the new `DebugOverlay` in too_runner
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
        self.nodes[id.0].as_mut().unwrap().view.inhabit(view);
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
        let parent = self.nodes[id.0].as_mut().unwrap();
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
        let id = ViewId(id);

        let parent = self.nodes[parent.0].as_mut().unwrap();
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

        let Some(node) = self.nodes.get_mut(id.0).and_then(<Option<_>>::as_mut) else {
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
            if let Some(node) = self.nodes.remove(id.0).flatten() {
                queue.extend(node.children);
                if let Some(parent) = node.parent {
                    if let Some(parent) = self.nodes.get_mut(parent.0).and_then(|s| s.as_mut()) {
                        parent.children.retain(|&child| child != id);
                    }
                }
            }
        }
    }

    fn cleanup(&mut self, start: ViewId) {
        let node = self.nodes[start.0].as_mut().unwrap();
        if node.next >= node.children.len() {
            return;
        }

        let children = &node.children[node.next..];
        let mut queue = VecDeque::from_iter(children.iter().copied());
        self.removed.extend_from_slice(children);
        node.children.truncate(node.next);

        while let Some(id) = queue.pop_front() {
            self.removed.push(id);
            let Some(next) = self.nodes.remove(id.0).flatten() else {
                unreachable!("child: {id:?} should exist for {start:?}")
            };
            queue.extend(next.children);
        }
    }
}

mod debug_fmt {
    use std::fmt::{Debug, Formatter, Result};

    pub const fn str(s: &str) -> impl Debug + '_ {
        struct NoQuote<'a>(&'a str);
        impl<'a> Debug for NoQuote<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                f.write_str(self.0)
            }
        }
        NoQuote(s)
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
