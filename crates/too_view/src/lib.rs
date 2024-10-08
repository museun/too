#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
use std::{
    any::{Any, TypeId},
    collections::{HashMap, VecDeque},
};

use too::{animation::AnimationManager, Pixel, Rgba, Surface as TooSurface};

mod text;

mod debug;
pub use debug::{debug_flat_tree, debug_flow_tree};

pub mod geom;
use geom::{float_step_exclusive, Point, Rectf, Size, Space, Vector};

pub mod views;

mod erased_view;
use erased_view::{ErasedView, ViewMarker};

pub mod view;
use view::Context;
pub use view::{Args, View, ViewExt};

mod view_node;
use view_node::{NodeSlot, ViewNode};

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

pub trait WidthProperty: 'static {
    const WIDTH: f32;
}

pub trait HeightProperty: 'static {
    const HEIGHT: f32;
}

pub trait FilledProperty: 'static {
    const FILLED: char;
    const CROSS: char = Self::FILLED;
}

pub trait UnfilledProperty: 'static {
    const UNFILLED: char;
}

#[derive(Default)]
pub struct Properties {
    list: Vec<Box<dyn Any>>,
    local: HashMap<ViewId, Vec<Box<dyn Any>>>,
}

impl Properties {
    pub fn with<P: 'static>(mut self, item: P) -> Self {
        self.insert(item);
        self
    }

    pub fn with_default<P: 'static + Default>(mut self) -> Self {
        self.insert_default::<P>();
        self
    }
}

impl Properties {
    pub fn width<T: WidthProperty>(&mut self) -> f32 {
        struct Width<T: WidthProperty> {
            value: f32,
            _marker: std::marker::PhantomData<T>,
        }
        self.get_or_insert_with::<Width<T>>(|| Width {
            value: T::WIDTH,
            _marker: std::marker::PhantomData,
        })
        .value
    }

    pub fn height<T: HeightProperty>(&mut self) -> f32 {
        struct Height<T: HeightProperty> {
            value: f32,
            _marker: std::marker::PhantomData<T>,
        }
        self.get_or_insert_with::<Height<T>>(|| Height {
            value: T::HEIGHT,
            _marker: std::marker::PhantomData,
        })
        .value
    }

    pub fn filled<T: FilledProperty>(&mut self) -> char {
        struct Filled<T: FilledProperty> {
            value: char,
            _marker: std::marker::PhantomData<T>,
        }
        self.get_or_insert_with::<Filled<T>>(|| Filled {
            value: T::FILLED,
            _marker: std::marker::PhantomData,
        })
        .value
    }

    // TODO this is a bad name
    pub fn filled_cross<T: FilledProperty>(&mut self) -> char {
        struct FilledCross<T: FilledProperty> {
            value: char,
            _marker: std::marker::PhantomData<T>,
        }
        self.get_or_insert_with::<FilledCross<T>>(|| FilledCross {
            value: T::CROSS,
            _marker: std::marker::PhantomData,
        })
        .value
    }

    pub fn unfilled<T: UnfilledProperty>(&mut self) -> char {
        struct Unfilled<T: UnfilledProperty> {
            value: char,
            _marker: std::marker::PhantomData<T>,
        }
        self.get_or_insert_with::<Unfilled<T>>(|| Unfilled {
            value: T::UNFILLED,
            _marker: std::marker::PhantomData,
        })
        .value
    }

    pub fn flex(&mut self, id: ViewId) -> Option<views::Flex> {
        self.get_for(id).copied()
    }
}

impl Properties {
    pub fn clear_locals(&mut self) {
        self.local.clear();
    }

    pub fn remove_all_for_id(&mut self, id: ViewId) {
        self.local.remove(&id);
    }

    pub fn get_for<P: 'static>(&self, id: ViewId) -> Option<&P> {
        self.local
            .get(&id)?
            .iter()
            .find_map(|c| c.downcast_ref::<P>())
    }

    pub fn get_or_default_for<P: 'static + Default>(&mut self, id: ViewId) -> &P {
        self.get_or_insert_with_for(P::default, id)
    }

    pub fn get_or_insert_for<P: 'static>(&mut self, value: P, id: ViewId) -> &P {
        self.get_or_insert_with_for(|| value, id)
    }

    pub fn get_or_insert_with_for<P: 'static>(
        &mut self,
        value: impl FnOnce() -> P,
        id: ViewId,
    ) -> &P {
        let Some(index) = self.get_index_for::<P>(id) else {
            let item = value();
            self.insert(item);
            return self.local[&id].last().unwrap().downcast_ref::<P>().unwrap();
        };
        self.local[&id][index].downcast_ref::<P>().unwrap()
    }

    pub fn insert_for<P: 'static + std::fmt::Debug>(&mut self, item: P, id: ViewId) {
        match self.get_index_for::<P>(id) {
            Some(index) => self.local.entry(id).or_default()[index] = Box::new(item),
            None => self.local.entry(id).or_default().push(Box::new(item)),
        }
    }

    pub fn insert_default_for<P: 'static + Default + std::fmt::Debug>(&mut self, id: ViewId) {
        self.insert_for(P::default(), id);
    }

    fn get_index_for<P: 'static>(&self, id: ViewId) -> Option<usize> {
        self.local.get(&id)?.iter().position(|item| item.is::<P>())
    }
}

impl Properties {
    pub fn insert<P: 'static>(&mut self, item: P) {
        match self.get_index::<P>() {
            Some(index) => self.list[index] = Box::new(item),
            None => self.list.push(Box::new(item)),
        }
    }

    pub fn insert_default<P: 'static + Default>(&mut self) {
        self.insert(P::default());
    }

    pub fn get<P: 'static>(&self) -> Option<&P> {
        self.list.iter().find_map(|c| c.downcast_ref::<P>())
    }

    pub fn get_or_default<P: 'static + Default>(&mut self) -> &P {
        self.get_or_insert_with(P::default)
    }

    pub fn get_or_insert<P: 'static>(&mut self, value: P) -> &P {
        self.get_or_insert_with(|| value)
    }

    pub fn get_or_insert_with<P: 'static>(&mut self, value: impl FnOnce() -> P) -> &P {
        let Some(index) = self.get_index::<P>() else {
            let item = value();
            self.insert(item);
            return self.list.last().unwrap().downcast_ref::<P>().unwrap();
        };

        self.list[index].downcast_ref::<P>().unwrap()
    }

    pub fn remove<P: 'static>(&mut self) -> bool {
        let len = self.list.len();
        self.list.retain(|c| !c.is::<P>());
        len != self.list.len()
    }

    fn get_index<P: 'static>(&self) -> Option<usize> {
        self.list.iter().position(|item| item.is::<P>())
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

impl From<ViewId> for too::Index<ViewId> {
    fn from(value: ViewId) -> Self {
        too::Index::from_raw(value.0.to_bits())
    }
}

impl std::fmt::Debug for ViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}v{}", self.0.slot(), self.0.generation())
    }
}

mod input;
use input::Input;
pub use input::{Event, EventCtx, Handled, Interest};

pub struct AnimateCtx<'a, 'c, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub state: &'a mut T,
    pub too_ctx: too::Context<'c>,
    // TODO this needs the rect (but is it valid here?)
    nodes: &'a mut thunderdome::Arena<Node<T>>,
}

impl<'a, 'c, T: 'static> AnimateCtx<'a, 'c, T> {
    pub fn animate(&mut self, id: ViewId, dt: f32) {
        let Some(node) = self.nodes.get_mut(id.0) else {
            return;
        };

        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", id)
        };

        let ctx = AnimateCtx {
            current_id: id,
            children: &node.children,
            state: self.state,
            too_ctx: too::Context {
                overlay: self.too_ctx.overlay,
                commands: self.too_ctx.commands,
                size: self.too_ctx.size,
                animations: self.too_ctx.animations,
            },
            nodes: self.nodes,
        };

        node.view.animate(ctx, dt);
        self.nodes[id.0].inhabit(node);
    }
}

pub struct UpdateCtx<'a, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub state: &'a mut T,
    pub properties: &'a mut Properties,
    debug: &'a mut Vec<String>,
}

pub struct LayoutCtx<'a, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub state: &'a mut T,
    pub properties: &'a mut Properties,

    client_rect: Rectf,
    input: &'a mut Input,
    nodes: &'a mut thunderdome::Arena<Node<T>>,
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

        node.rect = Rectf::from(size.clamp(Size::ZERO, self.client_rect.size()));
        node.interest = interest;

        assert_eq!(Some(child), self.stack.pop());
        self.nodes[child.0].inhabit(node);

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
            node.rect += offset.into();
        }
    }

    pub fn translate_size(&mut self, child: ViewId, size: impl Into<Size>) {
        if let Some(node) = self.nodes.get_mut(child.0) {
            node.rect += size.into()
        }
    }

    pub fn debug(&mut self, msg: impl ToString) {
        self.debug.push(msg.to_string());
    }
}

pub struct Surface<'a> {
    rect: Rectf,
    surface: &'a mut TooSurface,
}

impl<'a> Surface<'a> {
    pub fn surface_raw(&mut self) -> &mut TooSurface {
        self.surface
    }

    pub const fn rect(&self) -> Rectf {
        self.rect
    }

    pub fn horizontal_fill(&mut self, vec: impl Into<Vector>, pixel: impl Into<Pixel>) {
        let vec = vec.into().round();
        let pixel = pixel.into();
        for x in float_step_exclusive(vec.x, vec.y, 1.0) {
            self.set((x, 0.0), pixel);
        }
    }

    pub fn vertical_fill(&mut self, vec: impl Into<Vector>, pixel: impl Into<Pixel>) {
        let vec = vec.into().round();
        let pixel = pixel.into();
        for y in float_step_exclusive(vec.x, vec.y, 1.0) {
            self.set((0.0, y), pixel);
        }
    }

    pub fn fill(&mut self, pixel: impl Into<Pixel>) {
        let pixel = pixel.into();
        let vec = Vector::from((self.rect.width(), self.rect.height())).round();
        for y in float_step_exclusive(0.0, vec.y, 1.0) {
            for x in float_step_exclusive(0.0, vec.x, 1.0) {
                self.set((x, y), pixel);
            }
        }
    }

    pub fn set(&mut self, point: impl Into<Point>, pixel: impl Into<Pixel>) {
        let point = point.into() + self.rect.left_top().to_vector();
        let pos = point.into();
        self.surface.set(pos, pixel.into());
    }
}

pub struct DrawCtx<'a, 't, T: 'static> {
    pub rect: Rectf,
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub surface: Surface<'t>,
    pub state: &'a mut T,
    pub theme: &'a Theme,
    pub properties: &'a mut Properties,
    pub too_ctx: too::Context<'t>,

    nodes: &'a mut thunderdome::Arena<Node<T>>,
    stack: &'a mut Vec<ViewId>,
    debug: &'a mut Vec<String>,
}

impl<'a, 'c: 't, 't, T: 'static> DrawCtx<'a, 't, T> {
    pub fn draw(&mut self, id: ViewId) {
        let Some(node) = self.nodes.get_mut(id.0) else {
            return;
        };

        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", id)
        };

        self.stack.push(id);

        let ctx = DrawCtx {
            rect: node.rect,
            current_id: id,
            children: &node.children,
            surface: Surface {
                rect: node.rect,
                surface: self.surface.surface,
            },
            too_ctx: too::Context {
                overlay: self.too_ctx.overlay,
                commands: self.too_ctx.commands,
                size: self.too_ctx.size,
                animations: self.too_ctx.animations,
            },
            state: self.state,
            theme: self.theme,
            properties: self.properties,
            nodes: self.nodes,
            stack: self.stack,
            debug: self.debug,
        };

        node.view.draw(ctx);
        assert_eq!(Some(id), self.stack.pop());
        self.nodes[id.0].inhabit(node);
    }

    pub fn animations(&mut self) -> &mut AnimationManager {
        self.too_ctx.animations
    }

    pub fn debug(&mut self, msg: impl ToString) {
        self.debug.push(msg.to_string());
    }
}

enum Either<L, R> {
    Left(L),
    Right(R),
}

#[derive(Copy, Clone, Default)]
enum Toggle {
    Yes,
    #[default]
    No,
}

pub(crate) type Node<T> = NodeSlot<ViewNode<T>>;

pub struct Ui<T: 'static> {
    nodes: thunderdome::Arena<Node<T>>,
    root: ViewId,

    input: Input,

    stack: Vec<ViewId>,
    removed: Vec<ViewId>,

    light_mode: bool,
    theme: Theme,
    properties: Properties,

    rect: Rectf,
    quit: bool,

    debug: Vec<String>,

    toggle_fps: Toggle,
    toggle_debug: Toggle,
    //
    // TODO reuse vecdeque from the BFS
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

    pub fn toggle_fps(&mut self) {
        self.toggle_fps = Toggle::Yes
    }

    pub fn toggle_debug(&mut self) {
        self.toggle_debug = Toggle::Yes
    }

    pub fn current(&self) -> ViewId {
        self.stack.last().copied().unwrap_or(self.root())
    }

    pub fn light_mode(&mut self) -> &mut bool {
        &mut self.light_mode
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
            root: ViewId(nodes.insert(NodeSlot::Occupied(ViewNode::occupied(views::RootView)))),
            nodes,

            stack: Vec::new(),
            removed: Vec::new(),

            input: Input::default(),

            theme: Theme::dark(),
            properties,

            rect: rect.into(),
            quit: false,

            light_mode: false,

            debug: Vec::new(),

            toggle_debug: Toggle::No,
            toggle_fps: Toggle::No,
        }
    }

    fn scope(&mut self, state: &mut T, apply: fn(&mut Context<T>), mut ctx: too::Context) {
        self.begin();

        apply(&mut Context {
            ui: self,
            state,
            animations: ctx.animations_mut(),
        });

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

    fn animate(&mut self, state: &mut T, dt: f32, too_ctx: too::Context) {
        let node = &mut self.nodes[self.root.0];
        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", self.root)
        };

        let ctx = AnimateCtx::<T> {
            current_id: self.root,
            children: &node.children,
            state,
            too_ctx,
            nodes: &mut self.nodes,
        };

        node.view.animate(ctx, dt);
        self.nodes[self.root.0].inhabit(node);
    }

    fn event(&mut self, state: &mut T, event: too::Event, too_ctx: too::Context) {
        if let too::Event::Resize(size) = event {
            self.rect = Rectf::min_size(Point::ZERO, size.into());
        }

        self.input.handle(
            &event, //
            &mut self.nodes,
            state,
            too_ctx,
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
            min: self.rect.size(),
            max: self.rect.size(),
        };

        let _ = node.view.layout(ctx, space);
        self.nodes[self.root.0].inhabit(node);
    }

    fn render(&mut self, state: &mut T, surface: &mut TooSurface, mut too_ctx: too::Context) {
        let node = &mut self.nodes[self.root.0];

        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", self.root)
        };

        let mut surface = Surface {
            rect: surface.rect().into(),
            surface,
        };

        surface.fill(self.theme.background);

        let too_ctx_two = too::Context {
            overlay: too_ctx.overlay,
            commands: too_ctx.commands,
            size: too_ctx.size,
            animations: too_ctx.animations,
        };

        let ctx = DrawCtx::<T> {
            rect: surface.rect(),
            current_id: self.root,
            children: &node.children,
            surface,
            too_ctx: too_ctx_two,
            state,
            theme: &self.theme,
            properties: &mut self.properties,
            nodes: &mut self.nodes,
            stack: &mut self.stack,
            debug: &mut self.debug,
        };
        node.view.draw(ctx);
        self.nodes[self.root.0].inhabit(node);

        if let Toggle::Yes = std::mem::take(&mut self.toggle_fps) {
            too_ctx.overlay().fps.toggle();
        }

        if let Toggle::Yes = std::mem::take(&mut self.toggle_debug) {
            too_ctx.overlay().debug.toggle();
        }

        too_ctx.overlay().debug.extend(self.debug.drain(..));
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
            debug: &mut self.debug,
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
            self.removed.push(id);
            if let Some(node) = self.nodes.remove(id.0) {
                queue.extend(&node.children);
                if let Some(parent) = node.parent {
                    if let Some(parent) = self.nodes.get_mut(parent.0).map(|s| s.as_mut()) {
                        parent.children.retain(|&child| child != id);
                    }
                }
            }
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
