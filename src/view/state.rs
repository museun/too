use std::cell::{Ref, RefCell};

use compact_str::{CompactString, ToCompactString};

use crate::{math::Rect, rasterizer::Rasterizer, Animations, Str};

use super::{
    helpers::Queue, input::InputState, render::RenderNodes, style::Palette, ui::Ui, LayoutNode,
    LayoutNodes, ViewId, ViewNodes,
};

pub struct State {
    pub(in crate::view) nodes: ViewNodes,
    pub(in crate::view) layout: LayoutNodes,
    pub(in crate::view) render: RenderNodes,
    pub(in crate::view) input: InputState,
    pub(in crate::view) animations: Animations,
    pub(in crate::view) palette: RefCell<Palette>,
    pub(in crate::view) frame_count: u64,
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
            palette: RefCell::new(palette),
            frame_count: 0,
        }
    }

    pub fn debug(&self, msg: impl Into<Str>) {
        if matches!(DEBUG.with(|c| c.mode.get()), DebugMode::Off) {
            return;
        }
        let msg = msg.into();
        let msg = msg.trim();
        if msg.is_empty() {
            return;
        }
        debug(msg);
    }

    pub fn set_palette(&self, palette: Palette) {
        *self.palette.borrow_mut() = palette
    }

    pub fn palette(&self) -> Ref<'_, Palette> {
        self.palette.borrow()
    }

    pub fn set_debug_mode(&self, mode: DebugMode) {
        DEBUG.with(|c| c.mode.set(mode))
    }

    pub fn root(&self) -> ViewId {
        self.nodes.root()
    }

    pub fn current(&self) -> ViewId {
        self.nodes.current()
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    pub fn event(&mut self, event: &crate::Event) {
        if let crate::Event::Resize(size) = event {
            DEBUG.with(|c| c.queue.borrow_mut().resize(size.y as usize))
        }

        // TODO debounce 'event'
        let _resp = self
            .input
            .update(&self.nodes, &self.layout, &mut self.animations, event);
    }

    pub fn update(&mut self, dt: f32) {
        self.animations.update(dt);
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    pub fn build<R: 'static>(&mut self, rect: Rect, mut show: impl FnMut(&Ui) -> R) -> R {
        let root = self.nodes.root;
        self.layout.nodes[root].rect = rect;

        self.begin();
        let resp = show(&Ui::new(self, rect));
        self.end();

        self.layout.compute_all(&self.nodes, &self.input, rect);
        resp
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    pub fn render(&mut self, rasterizer: &mut impl Rasterizer) {
        self.frame_count += 1;

        let root = self.root();
        let rect = self.layout.rect(root).unwrap();
        rasterizer.clear(self.palette.get_mut().background);

        self.render.draw(
            root,
            &self.nodes,
            &self.layout,
            &self.input,
            self.palette.get_mut(),
            &mut self.animations,
            rasterizer,
            rect,
        );

        // DEBUG.with(|c| {
        //     let mut debug = c.queue.borrow_mut();
        //     if debug.is_empty() {
        //         return;
        //     }

        //     let mut layout = LinearLayout::vertical()
        //         .wrap(false)
        //         .anchor(Anchor2::LEFT_TOP)
        //         .layout(surface.rect());

        //     match c.mode.get() {
        //         DebugMode::PerFrame => {
        //             for msg in debug.drain() {
        //                 let text = Text::new(msg).fg(Rgba::hex("#F00")).bg(Rgba::hex("#000"));
        //                 if let Some(rect) = layout.allocate(text.size()) {
        //                     text.draw(rect, surface);
        //                 }
        //             }
        //         }
        //         DebugMode::Rolling => {
        //             for msg in debug.iter() {
        //                 let text = Text::new(msg).fg(Rgba::hex("#F00")).bg(Rgba::hex("#000"));
        //                 if let Some(rect) = layout.allocate(text.size()) {
        //                     text.draw(rect, surface);
        //                 }
        //             }
        //         }
        //         DebugMode::Off => {}
        //     }
        // });
    }

    fn begin(&mut self) {
        self.nodes.start();
        self.render.start();
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

#[derive(Debug, Default)]
pub(in crate::view) struct Debug {
    queue: RefCell<Queue<CompactString>>,
    pub(in crate::view) mode: std::cell::Cell<DebugMode>,
}

thread_local! {
    static DEBUG: Debug = const { Debug::new() }
}

pub fn debug(msg: impl Into<Str>) {
    DEBUG.with(|c| c.push(msg.into().0))
}

impl Debug {
    const fn new() -> Self {
        Self {
            queue: RefCell::new(Queue::new(25)),
            mode: std::cell::Cell::new(DebugMode::Rolling),
        }
    }

    pub(in crate::view) fn for_each(mut f: impl FnMut(&str)) {
        DEBUG.with(|c| {
            for msg in c.queue.borrow().iter() {
                f(msg);
            }
        })
    }

    fn push(&self, msg: impl ToCompactString) {
        if matches!(self.mode.get(), DebugMode::Off) {
            return;
        }
        let msg = msg.to_compact_string();
        let msg = msg.trim();
        if msg.is_empty() {
            return;
        }
        self.queue.borrow_mut().push(msg.into());
    }

    fn iter(&mut self) -> impl ExactSizeIterator<Item = &str> + use<'_> {
        self.queue.get_mut().iter().map(<_>::as_ref)
    }
}
