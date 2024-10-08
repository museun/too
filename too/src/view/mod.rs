#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
use std::{
    any::TypeId,
    collections::VecDeque,
    marker::PhantomData,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{
    animation::AnimationManager, overlay::Overlay, Backend, Commands, EventReader, MouseButton,
    Surface as TooSurface, TermRenderer,
};

mod text;

mod debug;
pub use debug::{debug_flat_tree, debug_flow_tree};

pub mod geom;
use geom::{Point, Rectf, Space};

pub mod views;

mod erased_view;
use erased_view::{ErasedView, ViewMarker};

pub mod view;
use view::Context;
pub use view::{Args, View, ViewExt};

mod view_node;
use view_node::{NodeSlot, ViewNode};

pub mod elements;

pub mod properties;
pub use properties::Properties;

mod theme;
pub use theme::Theme;

pub mod debug_fmt;

mod input;
use input::Input;
pub use input::{
    Event, EventCtx, Handled, Interest, KeyInput, MouseClick, MouseDragHeld, MouseDragRelease,
    MouseDragStart, MouseHeld, MouseMove, MouseScroll,
};

mod destroy_ctx;
pub use destroy_ctx::DestroyCtx;

mod animate_ctx;
pub use animate_ctx::AnimateCtx;

mod update_ctx;
pub use update_ctx::UpdateCtx;

mod layout_ctx;
pub use layout_ctx::LayoutCtx;

mod draw_ctx;
pub use draw_ctx::{DrawCtx, Surface};

pub trait App: Sized + 'static {
    fn view(ctx: &mut Context<Self>);
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ViewId(thunderdome::Index);

impl From<ViewId> for crate::Index<ViewId> {
    fn from(value: ViewId) -> Self {
        crate::Index::from_raw(value.0.to_bits())
    }
}

impl std::fmt::Debug for ViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}v{}", self.0.slot(), self.0.generation())
    }
}

pub(crate) type Node<T> = NodeSlot<ViewNode<T>>;

pub struct Ui<T: 'static> {
    nodes: thunderdome::Arena<Node<T>>,
    root: ViewId,

    input: Input,

    stack: Vec<ViewId>,
    removed: Vec<ViewId>,

    light_mode: Rc<std::cell::Cell<bool>>,
    theme: Theme,
    properties: Properties,

    rect: Rectf,
    quit: bool,

    animations: AnimationManager,
    overlay: Overlay,
    commands: Commands,
    //
    // TODO reuse vecdeque from the BFS
}

impl<T: App> Ui<T> {}

impl<T: 'static> Default for Ui<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> Ui<T> {
    pub fn new() -> Self {
        let mut nodes = thunderdome::Arena::new();
        Self {
            root: ViewId(nodes.insert(NodeSlot::Occupied(ViewNode::occupied(views::RootView)))),
            nodes,

            stack: Vec::new(),
            removed: Vec::new(),

            input: Input::default(),

            theme: Theme::dark(),
            properties: Properties::default(),

            animations: AnimationManager::default(),
            overlay: Overlay::default(),
            commands: Commands::default(),

            rect: Rectf::ZERO,
            quit: false,

            light_mode: Rc::new(std::cell::Cell::new(false)),
        }
    }

    pub fn with_overlay_settings(mut self, settings: impl Fn(&mut Overlay)) -> Self {
        settings(&mut self.overlay);
        self
    }

    pub fn with_properties(mut self, properties: Properties) -> Self {
        self.properties = properties;
        self
    }

    pub fn run(mut self, mut app: T, mut term: impl Backend + EventReader) -> std::io::Result<()>
    where
        T: App,
    {
        const MIN_UPS: f32 = 10.0;
        const MAX_UPS: f32 = 60.0;

        let mut surface = crate::Surface::new(term.size());
        self.rect = surface.rect().into();

        let mut target_ups = MAX_UPS;
        let mut base_target = Duration::from_secs_f32(1.0 / target_ups);
        let mut prev = Instant::now();

        let mut buffered_events = VecDeque::new();

        loop {
            let frame_start = Instant::now();

            let mut event_dur = Duration::ZERO;
            while let Some(ev) = term.try_read_event() {
                if ev.is_quit() {
                    return Ok(());
                }

                let start = Instant::now();
                // TODO collapse resizes
                surface.update(&ev);

                buffered_events.push_back(ev);
                event_dur += start.elapsed();

                // only spend up to half of the budget on reading events
                if event_dur >= base_target / 2 {
                    break;
                }
            }

            for cmd in self.commands.drain() {
                term.command(cmd);
            }

            let mut accum = frame_start.duration_since(prev).as_secs_f32();
            let target_dur = base_target.as_secs_f32();
            while accum >= target_dur {
                let start = Instant::now();
                self.animate(&mut app, target_dur);
                self.animations.update(target_dur);

                let update_dur = start.elapsed().as_secs_f32();
                accum -= target_dur;

                target_ups = if update_dur > target_dur {
                    (target_ups * 0.9).max(MIN_UPS)
                } else {
                    (target_ups * 10.05).min(MAX_UPS)
                }
            }

            self.rect = surface.rect().into();
            self.scope(&mut app, buffered_events.drain(..)); // update happens here

            if term.should_draw() {
                self.render(&mut app, &mut surface);
                Overlay::default_draw(&mut self.overlay, &mut surface);
                surface.render(&mut TermRenderer::new(&mut term))?;
            }

            let total = frame_start.elapsed() - event_dur;
            if let Some(sleep) = base_target
                .checked_sub(total)
                .filter(|&d| d > Duration::ZERO)
            {
                std::thread::sleep(sleep);
            }

            self.overlay
                .fps
                .push(frame_start.duration_since(prev).as_secs_f32());

            prev = frame_start;
            base_target = Duration::from_secs_f32(1.0 / target_ups)
        }
    }

    pub fn ctx<'a, 'b>(&'a mut self, state: &'b mut T) -> Context<'a, 'b, T> {
        Context { ui: self, state }
    }

    pub fn request_quit(&mut self) {
        self.quit = true
    }

    pub fn root(&self) -> ViewId {
        self.root
    }

    pub const fn properties(&self) -> &Properties {
        &self.properties
    }

    pub fn properties_mut(&mut self) -> &mut Properties {
        &mut self.properties
    }

    pub const fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn theme_mut(&mut self) -> &mut Theme {
        &mut self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) -> Theme {
        std::mem::replace(&mut self.theme, theme)
    }

    pub fn toggle_fps(&mut self) {
        self.overlay.fps.toggle();
    }

    pub fn toggle_debug(&mut self) {
        self.overlay.debug.toggle();
    }

    pub fn current(&self) -> ViewId {
        self.stack.last().copied().unwrap_or(self.root())
    }

    pub fn debug(&mut self, msg: impl ToString) {
        self.overlay.debug.push(msg.to_string());
    }

    pub fn primary_button_state(&self) -> input::ButtonState {
        self.input
            .mouse
            .buttons
            .get(&MouseButton::Primary)
            .copied()
            .unwrap_or(input::ButtonState::Released)
    }

    pub fn previous_mouse_pos(&self) -> Point {
        self.input.mouse.previous
    }

    pub fn current_mouse_pos(&self) -> Point {
        self.input.mouse.current
    }

    // pub fn client_rect(&self) -> Rectf {
    //     self.rect
    // }

    // pub fn available_rect(&self) -> Rectf {
    //     let id = self.current();
    //     self.nodes
    //         .get(id.0)
    //         .map(Option::as_ref)
    //         .flatten()
    //         .map(|c| c.rect)
    //         .unwrap_or(Rectf::ZERO)
    // }
}

impl<T: 'static> Ui<T> {
    fn scope(&mut self, state: &mut T, events: impl IntoIterator<Item = crate::Event>)
    where
        T: App,
    {
        for event in events {
            self.event(state, event);
            self.begin();
            T::view(&mut Context { ui: self, state });
            self.end(state);
        }

        self.begin();
        T::view(&mut Context { ui: self, state });
        self.end(state);
    }

    fn begin(&mut self) {
        self.nodes[self.root.0].as_mut().next = 0;
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
        let NodeSlot::Occupied(root) = &mut self.nodes[self.root.0] else {
            unreachable!("root node {:?} was not found", self.root);
        };
        root.rect = self.rect;

        let mut queue = VecDeque::from_iter([(self.root, Point::ZERO)]);
        while let Some((id, pos)) = queue.pop_front() {
            let Some(node) = self.nodes.get_mut(id.0) else {
                continue;
            };

            let offset = pos.to_vector();
            node.rect += offset;
            queue.extend(node.children.iter().map(|&id| (id, node.rect.min)))
        }
    }

    fn animate(&mut self, state: &mut T, dt: f32) {
        let node = &mut self.nodes[self.root.0];
        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", self.root)
        };

        let ctx = AnimateCtx::<T> {
            current_id: self.root,
            children: &node.children,
            state,
            animations: &mut self.animations,
            nodes: &mut self.nodes,
        };

        node.view.animate(ctx, dt);
        self.nodes[self.root.0].inhabit(node);
    }

    fn event(&mut self, state: &mut T, event: crate::Event) {
        if let crate::Event::Resize(size) = event {
            self.rect = Rectf::min_size(Point::ZERO, size.into());
        }

        self.input.handle(
            &event, //
            &mut self.nodes,
            state,
            &mut self.animations,
            &mut self.overlay,
            &mut self.commands,
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
            properties: &mut self.properties,
            input: &mut self.input,
            client_rect: self.rect,
            nodes: &mut self.nodes,
            stack: &mut self.stack,
            debug: &mut self.overlay.debug,
        };

        let space = Space {
            min: self.rect.size(),
            max: self.rect.size(),
        };

        let _ = node.view.layout(ctx, space);
        self.nodes[self.root.0].inhabit(node);
    }

    fn render(&mut self, state: &mut T, surface: &mut TooSurface) {
        let node = &mut self.nodes[self.root.0];

        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", self.root)
        };

        let mut surface = Surface {
            rect: surface.rect().into(),
            surface,
        };

        surface.fill(self.theme.background);

        let ctx = DrawCtx::<T> {
            rect: surface.rect(),
            current_id: self.root,
            children: &node.children,
            surface,
            animations: &mut self.animations,
            state,
            theme: &self.theme,
            properties: &mut self.properties,
            nodes: &mut self.nodes,
            stack: &mut self.stack,
            debug: &mut self.overlay.debug,
        };
        node.view.draw(ctx);
        self.nodes[self.root.0].inhabit(node);
    }

    fn begin_view<V>(&mut self, state: &mut T, args: V::Args<'_>) -> (ViewId, V::Response)
    where
        V: View<T> + 'static,
    {
        let parent = self.current();

        let (id, mut view) = self.patch_view::<V>(args.clone(), parent);
        self.stack.push(id);

        let Some(actual_view) = view.as_any_mut().downcast_mut::<V>() else {
            unreachable!(
                "expected to get view: {}, got {}",
                std::any::type_name::<V>(),
                view.type_name()
            )
        };

        let ctx = UpdateCtx {
            current_id: id,
            children: &self.nodes[id.0].as_ref().children,
            state,
            properties: &mut self.properties,
            debug: &mut self.overlay.debug,
        };

        let resp = actual_view.update(ctx, args);
        self.nodes[id.0].as_mut().view.inhabit(view);
        (id, resp)
    }

    fn end_view(&mut self, id: ViewId) {
        let Some(old) = self.stack.pop() else {
            unreachable!("called end view without an active view")
        };
        assert_eq!(id, old, "end view did not match input view");
        self.cleanup(id);
    }

    fn append_view(&mut self, id: ViewId) -> Option<ViewId> {
        let parent = self.nodes[id.0].as_mut();
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
        let node = NodeSlot::Occupied(ViewNode::empty(parent));
        let id = ViewId(self.nodes.insert(node));

        let parent = self.nodes[parent.0].as_mut();
        if parent.next < parent.children.len() {
            parent.children[parent.next] = id;
        } else {
            parent.children.push(id);
        }
        parent.next += 1;

        let view = V::create(args);
        (id, Box::new(ViewMarker::new(view)))
    }

    fn patch_view<V>(
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

        let node = self.nodes[id.0].as_mut();

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
            {
                let node = self.nodes[id.0].as_mut();
                node.view.destroy(DestroyCtx {
                    current_id: id,
                    children: &node.children,
                    animations: &mut self.animations,
                    properties: &mut self.properties,
                    _marker: PhantomData,
                });
            }

            self.removed.push(id);
            let Some(node) = self.nodes.remove(id.0) else {
                continue;
            };
            queue.extend(&node.children);

            let Some(parent) = node.parent else {
                continue;
            };
            let Some(parent) = self.nodes.get_mut(parent.0) else {
                continue;
            };

            parent.as_mut().children.retain(|&child| child != id);
        }
    }

    fn cleanup(&mut self, start: ViewId) {
        let node = self.nodes[start.0].as_mut();
        if node.next >= node.children.len() {
            return;
        }

        let children = &node.children[node.next..];
        let mut queue = VecDeque::from_iter(children.iter().copied());
        self.removed.extend_from_slice(children);
        node.children.truncate(node.next);

        while let Some(id) = queue.pop_front() {
            self.removed.push(id);
            let Some(next) = self.nodes.remove(id.0).filter(NodeSlot::is_occupied) else {
                unreachable!("child: {id:?} should exist for {start:?}")
            };
            queue.extend(&next.children);
        }
    }
}
