#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
use std::{
    any::{Any, TypeId},
    collections::VecDeque,
    ops::Deref,
};

use too::{
    layout::{Anchor2, LinearLayout},
    shapes::{Fill, Text},
    Rgba, SurfaceMut,
};

pub mod geom;
use geom::{Point, Rectf, Size, Space, Vector};

pub mod views;

mod response;
pub use response::Response;

mod erased_view;
use erased_view::{ErasedView, ViewMarker};

pub mod view;
use view::Context;
pub use view::{Args, NoArgs, NoResponse, View, ViewExt};

mod view_node;
use view_node::ViewNode;

mod app;
pub use app::{App, AppRunner};

// TODO Axis support for this
pub struct Elements;
impl Elements {
    pub const LARGE_RECT: char = '█';
    pub const MEDIUM_RECT: char = '■';
    pub const SMALL_RECT: char = '▮';

    pub const CIRCLE: char = '●';
    pub const DIAMOND: char = '◆';

    pub const HORIZONTAL_LINE: char = '─';
    pub const THICK_HORIZONTAL_LINE: char = '━';
    pub const DASH_HORIZONTAL_LINE: char = '╌';
    pub const THICK_DASH_HORIZONTAL_LINE: char = '╍';

    pub const VERTICAL_LINE: char = '│';
    pub const THICK_VERTICAL_LINE: char = '┃';
    pub const DASH_VERTICAL_LINE: char = '╎';
    pub const THICK_DASH_VERTICAL_LINE: char = '╏';
}

pub trait Property
where
    Self: std::any::Any,
    Self: Deref<Target = Self::Value>,
{
    type Value: 'static;

    fn new(value: Self::Value) -> Self
    where
        Self: Sized;

    fn get(&self) -> &Self::Value {
        self.deref()
    }
}

pub trait WidthProperty: 'static + Sized {
    const WIDTH: f32;
    fn width(width: f32) -> Width<Self> {
        <Width<Self>>::new(width)
    }
}

pub struct Width<T: WidthProperty> {
    pub(crate) width: f32,
    _marker: std::marker::PhantomData<T>,
}

impl<T: WidthProperty> Default for Width<T> {
    fn default() -> Self {
        Self {
            width: T::WIDTH,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: WidthProperty> Deref for Width<T> {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.width
    }
}

impl<T: WidthProperty> Property for Width<T> {
    type Value = f32;
    fn new(value: Self::Value) -> Self {
        Self {
            width: value,
            _marker: std::marker::PhantomData,
        }
    }
}

pub trait HeightProperty: 'static + Sized {
    const HEIGHT: f32;
    fn height(height: f32) -> Height<Self> {
        <Height<Self>>::new(height)
    }
}

pub struct Height<T: HeightProperty> {
    pub(crate) height: f32,
    _marker: std::marker::PhantomData<T>,
}

impl<T: HeightProperty> Default for Height<T> {
    fn default() -> Self {
        Self {
            height: T::HEIGHT,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: HeightProperty> Deref for Height<T> {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.height
    }
}

impl<T: HeightProperty> Property for Height<T> {
    type Value = f32;
    fn new(value: Self::Value) -> Self {
        Self {
            height: value,
            _marker: std::marker::PhantomData,
        }
    }
}

pub trait FilledProperty: 'static + Sized {
    const FILLED: char;
    fn filled(filled: char) -> Filled<Self> {
        <Filled<Self>>::new(filled)
    }
}

pub struct Filled<T: FilledProperty> {
    pub(crate) char: char,
    _marker: std::marker::PhantomData<T>,
}

impl<T: FilledProperty> Default for Filled<T> {
    fn default() -> Self {
        Self {
            char: T::FILLED,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: FilledProperty> Deref for Filled<T> {
    type Target = char;
    fn deref(&self) -> &Self::Target {
        &self.char
    }
}

impl<T: FilledProperty> Property for Filled<T> {
    type Value = char;
    fn new(value: Self::Value) -> Self {
        Self {
            char: value,
            _marker: std::marker::PhantomData,
        }
    }
}

pub trait UnfilledProperty: 'static + Sized {
    const UNFILLED: char;
    fn unfilled(unfilled: char) -> Unfilled<Self> {
        <Unfilled<Self>>::new(unfilled)
    }
}

pub struct Unfilled<T: UnfilledProperty> {
    pub(crate) char: char,
    _marker: std::marker::PhantomData<T>,
}

impl<T: UnfilledProperty> Default for Unfilled<T> {
    fn default() -> Self {
        Self {
            char: T::UNFILLED,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: UnfilledProperty> Deref for Unfilled<T> {
    type Target = char;
    fn deref(&self) -> &Self::Target {
        &self.char
    }
}

impl<T: UnfilledProperty> Property for Unfilled<T> {
    type Value = char;
    fn new(value: Self::Value) -> Self {
        Self {
            char: value,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct Knob(char);
impl Default for Knob {
    fn default() -> Self {
        Self::ROUND
    }
}

impl Knob {
    pub const LARGE: Self = Self(Elements::LARGE_RECT);
    pub const MEDIUM: Self = Self(Elements::MEDIUM_RECT);
    pub const SMALL: Self = Self(Elements::SMALL_RECT);
    pub const ROUND: Self = Self(Elements::CIRCLE);
    pub const DIAMOND: Self = Self(Elements::DIAMOND);
}

impl Property for Knob {
    type Value = char;
    fn new(value: Self::Value) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for Knob {
    type Target = char;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct Properties(Vec<Box<dyn Any>>);

impl Properties {
    pub fn with<P: Property>(mut self, item: P) -> Self {
        self.insert(item);
        self
    }

    pub fn with_default<P: Property + Default>(mut self) -> Self {
        self.insert_default::<P>();
        self
    }

    pub fn width<T>(&mut self) -> f32
    where
        T: WidthProperty,
    {
        *self.get_or_default::<Width<T>>()
    }

    pub fn height<T>(&mut self) -> f32
    where
        T: HeightProperty,
    {
        *self.get_or_default::<Height<T>>()
    }

    pub fn filled<T>(&mut self) -> char
    where
        T: FilledProperty,
    {
        *self.get_or_default::<Filled<T>>()
    }

    pub fn unfilled<T>(&mut self) -> char
    where
        T: UnfilledProperty,
    {
        *self.get_or_default::<Unfilled<T>>()
    }

    pub fn insert<P: Property>(&mut self, item: P) {
        match self.get_index::<P>() {
            Some(index) => self.0[index] = Box::new(item),
            None => self.0.push(Box::new(item)),
        }
    }

    pub fn insert_default<P: Property + Default>(&mut self) {
        self.insert(P::default());
    }

    pub fn get<P: Property>(&self) -> Option<&P::Value> {
        self.0
            .iter()
            .find_map(|c| c.downcast_ref::<P>().map(P::get))
    }

    pub fn get_or_default<P: Property + Default>(&mut self) -> &P::Value {
        self.get_or_insert_with(P::default)
    }

    pub fn get_or_insert<P: Property>(&mut self, value: P) -> &P::Value {
        self.get_or_insert_with(|| value)
    }

    pub fn get_or_insert_with<P: Property>(&mut self, value: impl FnOnce() -> P) -> &P::Value {
        let Some(index) = self.get_index::<P>() else {
            let item = value();
            self.insert(item);
            return self.0.last().unwrap().downcast_ref::<P>().unwrap().get();
        };

        self.0[index].downcast_ref::<P>().unwrap().get()
    }

    pub fn remove<P: Property>(&mut self) -> bool {
        let len = self.0.len();
        self.0.retain(|c| !c.is::<P>());
        len != self.0.len()
    }

    fn get_index<P: Property>(&self) -> Option<usize> {
        self.0.iter().position(|item| item.is::<P>())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Theme {
    pub background: Rgba,
    pub foreground: Rgba,
    pub surface: Rgba,
    pub outline: Rgba,
    pub contrast: Rgba,
    pub primary: Rgba,
    pub secondary: Rgba,
    pub accent: Rgba,
    pub danger: Rgba,
    pub success: Rgba,
    pub warning: Rgba,
    pub info: Rgba,
}

impl Theme {
    pub const BACKGROUND: Style<Rgba> = Style::new("theme.background");
    pub const FOREGROUND: Style<Rgba> = Style::new("theme.foreground");

    // cardinality for this
    pub const SURFACE: Style<Rgba> = Style::new("theme.surface");

    // low constrast for these
    pub const OUTLINE: Style<Rgba> = Style::new("theme.outline");
    pub const CONTRAST: Style<Rgba> = Style::new("theme.contrast");
    pub const PRIMARY: Style<Rgba> = Style::new("theme.primary");
    pub const SECONDARY: Style<Rgba> = Style::new("theme.secondary");
    pub const ACCENT: Style<Rgba> = Style::new("theme.accent");
    pub const DANGER: Style<Rgba> = Style::new("theme.danger");
    pub const SUCCESS: Style<Rgba> = Style::new("theme.success");
    pub const WARNING: Style<Rgba> = Style::new("theme.warning");
    pub const INFO: Style<Rgba> = Style::new("theme.info");
}

impl Theme {
    pub const fn light() -> Self {
        Self {
            background: Rgba::hex("#E0E0E0"),
            foreground: Rgba::hex("#000000"),
            surface: Rgba::hex("#A3A5A8"),
            outline: Rgba::hex("#9DA2A8"),
            contrast: Rgba::hex("#161616"),
            primary: Rgba::hex("#8175DF"),
            secondary: Rgba::hex("#B8A52D"),
            accent: Rgba::hex("#776BC2"),
            danger: Rgba::hex("#C7343B"),
            success: Rgba::hex("#33D17A"),
            warning: Rgba::hex("#F9F35F"),
            info: Rgba::hex("#0077C2"),
        }
    }

    pub const fn dark() -> Self {
        Self {
            background: Rgba::hex("#131313"),
            foreground: Rgba::hex("#FFFFFF"),
            surface: Rgba::hex("#343434"),
            outline: Rgba::hex("#4D4D4D"),
            contrast: Rgba::hex("#F9E9E9"),
            primary: Rgba::hex("#55B1F0"),
            secondary: Rgba::hex("#8C8BED"),
            accent: Rgba::hex("#F4A151"),
            danger: Rgba::hex("#F05D61"),
            success: Rgba::hex("#9AF07A"),
            warning: Rgba::hex("#F9F35F"),
            info: Rgba::hex("#6A7DDA"),
        }
    }
}

pub struct Style<T: ?Sized> {
    key: u64,
    _marker: std::marker::PhantomData<fn(&T)>,
}

impl<T: ?Sized> Style<T> {
    #[inline(always)]
    pub const fn new(key: &str) -> Self {
        Self {
            key: hash_fnv_1a(key.as_bytes()),
            _marker: std::marker::PhantomData,
        }
    }
}

#[inline(always)]
const fn hash_fnv_1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        index += 1;
    }
    hash
}

impl<T: ?Sized> Copy for Style<T> {}
impl<T: ?Sized> Clone for Style<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ViewId(thunderdome::Index);

impl std::fmt::Debug for ViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ViewId")
            .field(&self.0.slot())
            .field(&self.0.generation())
            .finish()
    }
}

mod input;
use input::Input;
pub use input::{Event, EventCtx, Handled, Interest};

pub struct UpdateCtx<'a, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub state: &'a mut T,

    debug: &'a mut Vec<String>,
}

pub struct LayoutCtx<'a, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub state: &'a mut T,
    pub properties: &'a mut Properties,

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
                properties: self.properties,
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

    pub fn translate_pos(&mut self, child: ViewId, offset: impl Into<Vector>) {
        if let Some(node) = self.nodes.get_mut(child.0) {
            let Some(node) = node else {
                unreachable!("node {child:?} is missing")
            };

            node.rect += offset.into();
        }
    }

    pub fn translate_size(&mut self, child: ViewId, size: impl Into<Size>) {
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
    pub theme: &'a Theme,
    pub properties: &'a mut Properties,

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
            theme: self.theme,
            properties: self.properties,
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

    theme: Theme,
    properties: Properties,

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

    pub fn current(&self) -> ViewId {
        self.stack.last().copied().unwrap_or(self.root())
    }

    pub fn debug(&mut self, msg: impl ToString) {
        self.debug.push(msg.to_string());
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
    fn new(rect: impl Into<Rectf>, properties: Properties) -> Self {
        let mut nodes = thunderdome::Arena::new();
        Self {
            root: ViewId(nodes.insert(Some(ViewNode::occupied(views::RootView)))),
            nodes,

            stack: Vec::new(),
            removed: Vec::new(),

            input: Input::default(),

            theme: Theme::dark(),
            properties,

            rect: rect.into(),
            quit: false,

            debug: Vec::new(),
        }
    }

    fn scope(&mut self, state: &mut T, apply: fn(&mut Context<'_, T>), ctx: too::Context) {
        self.begin();
        apply(&mut Context { ui: self, state });
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

    fn event(&mut self, state: &mut T, event: too::Event) {
        if let too::Event::Resize(size) = event {
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
            properties: &mut self.properties,
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

        surface.draw(Fill::new(self.theme.background));

        let ctx = DrawCtx::<T> {
            rect: surface.rect().into(),
            current_id: self.root,
            children: &node.children,
            surface: &mut surface,
            state,
            theme: &self.theme,
            properties: &mut self.properties,
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

        // is this clone required?
        let (id, mut view) = self.update_view::<V>(args.clone(), parent);
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
            children: &self.nodes[id.0].as_ref().unwrap().children,
            state,
            debug: &mut self.debug,
        };

        let resp = actual_view.update(ctx, args);
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

        let view = V::create(args);
        (id, Box::new(ViewMarker::new(view)))
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
