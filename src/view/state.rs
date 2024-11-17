use std::collections::VecDeque;

use compact_str::{CompactString, ToCompactString};

use crate::{
    animation::Animations,
    backend::Event,
    helpers::Queue,
    layout::{Anchor2, LinearAllocator, LinearLayout},
    lock::{Lock, Ref},
    math::{Rect, Vec2},
    renderer::{Rasterizer, TextShape},
    Str,
};

#[allow(deprecated)]
use super::measure_text;

use super::{
    input::InputState, render::RenderNodes, style::Palette, ui::Ui, Layer, LayoutNode, LayoutNodes,
    ViewId, ViewNodes,
};

// TODO what of this should actually be public?
/// State is the entire state for a [`Ui`]
///
/// An event loop is provided for the terminal with [`run`](crate::run()) and [`application`](crate::application)
///
/// You can have multiple state, they are built with a [`Rect`] as their constraining bounds.
///
/// ## State life cycles
/// ```rust,no_run
/// let mut state = State::new(Palette::default(), Animations::default());
///
/// // provide a way to read events from a backend
/// if let Some(event) = read_event() {
///     state.event(&event);
/// }
///
/// // then before building (creating/updating) the ui tree, drive any frame-specific timers forward:
/// state.update(frame_time.dt());
///
/// // then as many times as you want, build the Ui tree
/// state.build(some_rect, |ui|{
///     display_user_ui(ui);
/// });
///
/// // finally, at the end of a frame render it out
/// state.render(&mut my_rasterizer);
/// ```
/// ### Summary
/// you should process things in this order:
/// - events
/// - frame time state
/// - building the ui
/// - rendering
pub struct State {
    pub(in crate::view) nodes: ViewNodes,
    pub(in crate::view) layout: LayoutNodes,
    pub(in crate::view) render: RenderNodes,
    pub(in crate::view) input: InputState,
    pub(in crate::view) animations: Animations,
    pub(in crate::view) palette: Lock<Palette>,
    pub(in crate::view) frame_count: u64,
    pub(in crate::view) dt: f32,
    pub(in crate::view) size_changed: Option<Vec2>,
}

impl Default for State {
    fn default() -> Self {
        Self::new(Palette::default(), Animations::default())
    }
}

impl State {
    /// Create a new state with a [`Palette`] and provided [`Animations`] manager
    pub fn new(palette: Palette, animations: Animations) -> Self {
        let nodes = ViewNodes::new();
        let mut layout = LayoutNodes::new();
        layout.nodes.insert(nodes.root, LayoutNode::new(nodes.root));

        Self {
            nodes,
            layout,
            render: RenderNodes::new(),
            input: InputState::default(),
            animations,
            palette: Lock::new(palette),
            frame_count: 0,
            dt: 1.0,
            size_changed: None,
        }
    }

    /// Set the current palette
    pub fn set_palette(&self, palette: Palette) {
        *self.palette.borrow_mut() = palette
    }

    /// Gets the current palette
    pub fn palette(&self) -> Ref<'_, Palette> {
        self.palette.borrow()
    }

    /// Get the root id for the current State Ui tree
    pub fn root(&self) -> ViewId {
        self.nodes.root()
    }

    /// Process any [Event]s
    #[cfg_attr(feature = "profile", profiling::function)]
    pub fn event(&mut self, event: &Event) {
        if let Event::Resize(size) = event {
            if self.size_changed.get_or_insert(*size) == size {
                self.size_changed.take();
            }

            Debug::resize(size.y as usize);
        }

        // TODO debounce 'event'
        let _resp = self.input.update(
            &self.nodes, //
            &self.layout,
            &mut self.animations,
            event,
        );
    }

    /// Update any animations with the frame delta
    pub fn update(&mut self, dt: f32) {
        self.animations.update(dt);
        self.dt = dt;
    }

    /// Build the ui state contained in the provided [`Rect`]
    ///
    /// This takes in a closure and is the only way of getting access to the [`Ui`] type.
    ///
    /// You should do this after you've processed [events](State::event) and [driven](State::update) any animations forward.
    ///
    /// Once you build the state, you can [render](State::render) it
    #[cfg_attr(feature = "profile", profiling::function)]
    pub fn build<R: 'static>(&mut self, rect: Rect, mut show: impl FnMut(&Ui) -> R) -> R {
        let root = self.nodes.root;
        self.layout.nodes[root].rect = rect;

        self.begin();
        let resp = show(&Ui::new(self, rect));
        self.end();

        self.layout.compute_all(&self.nodes, &mut self.input, rect);
        resp
    }

    /// Render the current state to a [`Rasterizer`]
    #[cfg_attr(feature = "profile", profiling::function)]
    pub fn render(&mut self, rasterizer: &mut impl Rasterizer) {
        self.frame_count += 1;

        let root = self.root();
        let rect = self.layout.rect(root).unwrap();
        rasterizer.clear(self.palette.get_mut().background);

        let mut pending = VecDeque::new();

        self.render.draw(
            root,
            &self.nodes,
            &self.layout,
            &self.input,
            self.palette.get_mut(),
            &mut pending,
            &mut self.animations,
            rasterizer,
        );

        // XXX should this clear?
        for layer in [Layer::Middle, Layer::Top, Layer::Debug] {
            self.render.current_layer = layer;
            while let Some(id) = pending.pop_front() {
                self.render.draw(
                    id,
                    &self.nodes,
                    &self.layout,
                    &self.input,
                    self.palette.get_mut(),
                    &mut pending,
                    &mut self.animations,
                    rasterizer,
                );
                if pending.back() == Some(&id) {
                    break;
                }
            }
        }

        self.render_debug(rect, rasterizer);
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    fn render_debug(&self, rect: Rect, rasterizer: &mut impl Rasterizer) {
        Debug::with(|c| {
            let mut debug = c.queue.borrow_mut();
            if debug.is_empty() {
                return;
            }

            let mut layout = LinearLayout::vertical()
                .wrap(false)
                .anchor(*c.anchor.borrow())
                .layout(rect);

            match *c.mode.borrow() {
                DebugMode::PerFrame => {
                    for msg in debug.drain() {
                        if !Debug::render(rasterizer, &mut layout, &msg) {
                            break;
                        }
                    }
                }
                DebugMode::Rolling => {
                    for msg in debug.iter() {
                        if !Debug::render(rasterizer, &mut layout, msg) {
                            break;
                        }
                    }
                }
                DebugMode::Off => {}
            }
        });
    }

    fn begin(&mut self) {
        self.nodes.start();
        self.render.start();
        self.layout.begin();
        self.input.begin(
            &self.nodes, //
            &self.layout,
            &mut self.animations,
        );
    }

    fn end(&mut self) {
        for id in self.nodes.finish() {
            self.layout.nodes.remove(id);
        }
        self.input.end();
        self.layout.end();
    }
}

/// Controls the behavior of the [`struct@Debug`]  overlay
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum DebugMode {
    /// Should the queue be cleared every frame?
    PerFrame,
    #[default]
    /// Should the queue be retained across frames -- but rotated at the max screen height
    Rolling,
    /// Should the overlay do nothing?
    Off,
}

/// A debug overlay
///
/// When this is enabled, any [`debug()`] calls will be rendered ontop of everything else.
#[derive(Debug)]
pub struct Debug {
    // TODO this should all be in the same `Lock`
    queue: Lock<Queue<CompactString>>,
    mode: Lock<DebugMode>,
    anchor: Lock<Anchor2>,
}

// TODO this should be conditionally in a LazyLock or a ThreadLocalKey
#[cfg(not(feature = "sync"))]
thread_local! {
    static DEBUG: Debug = const { Debug::new() }
}

#[cfg(feature = "sync")]
static DEBUG: std::sync::LazyLock<Debug> = std::sync::LazyLock::new(Debug::new);

/// Prints a debug message to the debug overlay.
///
/// ### Note
/// - If the message is empty (or spaces) it will be ignored.
/// - The debug queue is bounded by a size, and it acts as a FIFO queue -- so the most recent messages should always be available.
///
/// Depending on the [`DebugMode`] debug messages may be recycled every frame.
///
/// You can change the behavior with [`Debug::set_debug_mode()`]
///
/// ### Performance
/// You should favor [`crate::format_str!`] over [`std::format!`] for this
pub fn debug(msg: impl Into<Str>) {
    let msg = msg.into();
    let msg = msg.trim();
    if msg.is_empty() {
        return;
    }
    Debug::with(|c| c.push(msg));
}

impl Debug {
    const fn new() -> Self {
        Self {
            queue: Lock::new(Queue::new(25)),
            mode: Lock::new(DebugMode::Rolling),
            anchor: Lock::new(Anchor2::RIGHT_TOP),
        }
    }

    pub(in crate::view) fn with<R: 'static>(f: impl FnOnce(&Debug) -> R) -> R {
        #[cfg(not(feature = "sync"))]
        return DEBUG.with(|debug| f(debug));
        #[cfg(feature = "sync")]
        return f(&DEBUG);
    }

    /// Visits each message currently in the queue
    pub fn for_each(mut f: impl FnMut(&str)) {
        Self::with(|c| {
            for msg in c.queue.borrow().iter() {
                f(msg);
            }
        });
    }

    /// Sets the mode for the queue from here until the next mode change
    ///
    /// | Mode | Effect |
    /// | --- | --- |
    /// | PerFrame | Reset the queue every frame |
    /// | Rolling | Rotates the buffer across frames |
    /// | Off | Disable all debug overlay messages |
    pub fn set_debug_mode(debug_mode: DebugMode) {
        Self::with(|c| *c.mode.borrow_mut() = debug_mode);
    }

    /// Set where the debug overlay should be drawn.
    ///
    /// See [`Anchor2`] for options
    pub fn set_debug_anchor(anchor: Anchor2) {
        Self::with(|c| *c.anchor.borrow_mut() = anchor);
    }

    /// Is the debug overlay enabled? (E.g. is it on?)
    pub fn is_enabled() -> bool {
        !matches!(Self::with(|c| *c.mode.borrow()), DebugMode::Off)
    }

    pub(crate) fn resize(size: usize) {
        Self::with(|c| c.queue.borrow_mut().resize(size));
    }

    fn render(rasterizer: &mut dyn Rasterizer, layout: &mut LinearAllocator, msg: &str) -> bool {
        let text = TextShape::new(msg).fg("#F00").bg("#000");
        #[allow(deprecated)]
        let size = Vec2::from(measure_text(&text.label));
        let Some(rect) = layout.allocate(size) else {
            return false;
        };
        rasterizer.set_rect(rect);
        rasterizer.text(text);
        true
    }

    fn push(&self, msg: impl ToCompactString) {
        if matches!(*self.mode.borrow(), DebugMode::Off) {
            return;
        }
        let msg = msg.to_compact_string();
        let msg = msg.trim();
        if msg.is_empty() {
            return;
        }
        self.queue.borrow_mut().push(msg.into());
    }
}
