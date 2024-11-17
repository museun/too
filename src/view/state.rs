use std::collections::VecDeque;

use compact_str::{CompactString, ToCompactString};

use crate::{
    backend::Event,
    layout::{Anchor2, LinearAllocator, LinearLayout},
    lock::{Lock, Ref},
    math::{Rect, Vec2},
    rasterizer::Rasterizer,
    Animations, Str, TextShape,
};

use super::{
    helpers::Queue, input::InputState, render::RenderNodes, style::Palette, ui::Ui, Layer,
    LayoutNode, LayoutNodes, ViewId, ViewNodes,
};

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

    pub fn set_palette(&self, palette: Palette) {
        *self.palette.borrow_mut() = palette
    }

    pub fn palette(&self) -> Ref<'_, Palette> {
        self.palette.borrow()
    }

    pub fn root(&self) -> ViewId {
        self.nodes.root()
    }

    pub fn current(&self) -> ViewId {
        self.nodes.current()
    }

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

    pub fn update(&mut self, dt: f32) {
        self.animations.update(dt);
        self.dt = dt;
    }

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

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum DebugMode {
    PerFrame,
    #[default]
    Rolling,
    Off,
}

#[derive(Debug)]
pub(crate) struct Debug {
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

    pub(in crate::view) fn for_each(mut f: impl FnMut(&str)) {
        Self::with(|c| {
            for msg in c.queue.borrow().iter() {
                f(msg);
            }
        });
    }

    pub fn set_debug_mode(debug_mode: DebugMode) {
        Self::with(|c| *c.mode.borrow_mut() = debug_mode);
    }

    pub fn set_debug_anchor(anchor: Anchor2) {
        Self::with(|c| *c.anchor.borrow_mut() = anchor);
    }

    pub fn is_enabled() -> bool {
        !matches!(Self::with(|c| *c.mode.borrow()), DebugMode::Off)
    }

    pub(in crate::view) fn resize(size: usize) {
        Self::with(|c| c.queue.borrow_mut().resize(size));
    }

    fn render(rasterizer: &mut dyn Rasterizer, layout: &mut LinearAllocator, msg: &str) -> bool {
        let text = TextShape::new(msg).fg("#F00").bg("#000");
        #[allow(deprecated)]
        let size = Vec2::from(crate::measure_text(&text.label));
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
