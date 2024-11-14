use std::time::{Duration, Instant};

pub mod debug;
pub mod helpers;

mod state;
pub use state::{debug, DebugMode, State};

mod response;
pub use response::Response;

mod input;
pub use input::{EventCtx, Handled, Interest, ViewEvent};

mod ui;
pub use ui::Ui;

mod layout;
pub use layout::{IntrinsicSize, Layout, LayoutNode, LayoutNodes};

mod render;
pub use render::{CroppedSurface, Render};

mod view_nodes;
pub use view_nodes::{ViewNode, ViewNodes};

mod style;
pub use style::{Elements, Knob, Palette, StyleKind};

mod internal_views;

slotmap::new_key_type! {
    pub struct ViewId;
}

use crate::{
    layout::{Axis, Flex},
    math::{Size, Space},
    Animations,
};

pub trait Adhoc<'v>: Sized {
    type Output: 'static;
    fn show(self, ui: &Ui) -> Self::Output;
}

impl<'v, T> Adhoc<'v> for T
where
    T: Builder<'v>,
{
    type Output = Response<<T::View as View>::Response>;
    fn show(self, ui: &Ui) -> Self::Output {
        <T as ViewExt>::show(self, ui)
    }
}

pub trait Builder<'v>: Sized {
    type View: View<Args<'v> = Self>;
}

pub trait ViewExt<'v>: Builder<'v> {
    fn show(self, ui: &Ui) -> Response<<Self::View as View>::Response> {
        ui.show(self)
    }

    fn show_children<R>(
        self,
        ui: &Ui,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<(<Self::View as View>::Response, R)>
    where
        R: 'static,
    {
        ui.show_children(self, show)
    }
}

impl<'v, T> ViewExt<'v> for T where T: Builder<'v> {}

#[allow(unused_variables)]
pub trait View: Sized + 'static + std::fmt::Debug {
    type Args<'v>;
    type Response: 'static + Default;

    fn create(args: Self::Args<'_>) -> Self;

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        *self = Self::create(args);
        Self::Response::default()
    }

    fn flex(&self) -> Flex {
        Flex::Loose(0.0)
    }

    fn interests(&self) -> Interest {
        Interest::NONE
    }

    fn primary_axis(&self) -> Axis {
        Axis::Horizontal
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        self.default_event(event, ctx)
    }

    fn size(&self, intrinsic: IntrinsicSize, axis: Axis, extent: f32) -> f32 {
        let node = intrinsic.nodes.get_current();
        let mut size = 0.0_f32;
        for &child in &node.children {
            size = size.max(intrinsic.size(child, axis, extent))
        }
        size
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        self.default_layout(layout, space)
    }

    fn draw(&mut self, render: Render) {
        self.default_draw(render)
    }

    fn default_event(&mut self, event: ViewEvent, mut ctx: EventCtx) -> Handled {
        let node = ctx.nodes.get_current();
        let mut resp = Handled::Bubble;
        for &child in &node.children {
            let new = ctx.event(child, event);
            if new.is_sink() {
                return new;
            }
            resp = new;
        }
        resp
    }

    fn default_layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let current = layout.nodes.get_current();
        let mut size = Size::ZERO;
        for &child in &current.children {
            size = size.max(layout.compute(child, space))
        }
        size
    }

    fn default_draw(&mut self, mut render: Render) {
        let current = render.nodes.get_current();
        for &child in &current.children {
            render.draw(child)
        }
    }
}

pub trait Erased: std::any::Any + std::fmt::Debug {
    fn interests(&self) -> Interest;

    fn flex(&self) -> Flex;

    fn size(&self, size: IntrinsicSize, axis: Axis, extent: f32) -> f32;
    fn primary_axis(&self) -> Axis;

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled;
    fn layout(&mut self, layout: Layout, space: Space) -> Size;
    fn draw(&mut self, render: Render);

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_mut_any(&mut self) -> &mut dyn std::any::Any;
    fn type_name(&self) -> &'static str;
}

impl<T: View> Erased for T {
    #[inline(always)]
    fn interests(&self) -> Interest {
        T::interests(self)
    }

    #[inline(always)]
    fn flex(&self) -> Flex {
        T::flex(self)
    }

    #[inline(always)]
    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        T::event(self, event, ctx)
    }

    #[inline(always)]
    fn size(&self, size: IntrinsicSize, axis: Axis, extent: f32) -> f32 {
        T::size(self, size, axis, extent)
    }

    #[inline(always)]
    fn primary_axis(&self) -> Axis {
        T::primary_axis(self)
    }

    #[inline(always)]
    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        T::layout(self, layout, space)
    }

    #[inline(always)]
    fn draw(&mut self, render: Render) {
        T::draw(self, render)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

pub fn debug_view(mut app: impl FnMut(&Ui)) -> std::io::Result<()> {
    let s = debug::pretty_tree(|ui| app(ui));
    println!("{s}");
    for debug in s.debug() {
        println!("{debug}")
    }
    Ok(())
}

pub fn run<R: 'static>(app: impl FnMut(&Ui) -> R) -> std::io::Result<()> {
    application(Config::default(), app)
}

pub struct Config {
    pub palette: Palette,
    pub debug: DebugMode,
    pub animation: Animations,
    pub fps: f32,
    pub ctrl_c_quits: bool,
    pub ctrl_z_switches: bool,
    pub hook_panics: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            palette: Palette::dark(),
            debug: DebugMode::PerFrame,
            animation: Animations::default(),
            fps: 60.0,
            ctrl_c_quits: true,
            ctrl_z_switches: false,
            hook_panics: false,
        }
    }
}

pub fn application<R: 'static>(
    config: Config,
    mut app: impl FnMut(&Ui) -> R,
) -> std::io::Result<()> {
    use crate::{
        term::{Config as TermConfig, Term},
        Backend, EventReader, Surface,
    };

    let mut term = Term::setup(
        TermConfig::default()
            .hook_panics(config.hook_panics)
            .ctrl_c_quits(config.ctrl_c_quits)
            .ctrl_z_switches(config.ctrl_z_switches),
    )?;
    let mut surface = Surface::new(term.size());

    let mut state = State::new(config.palette, config.animation);
    state.set_debug_mode(config.debug);

    let target = Duration::from_secs_f32(1.0 / config.fps.max(1.0));
    let max_budget = (target / 2).max(Duration::from_millis(1));

    let mut prev = Instant::now();

    'outer: loop {
        #[cfg(feature = "profile")]
        {
            profiling::finish_frame!();
        }

        let mut should_render = false;
        let mut last_resize = None;

        let start = Instant::now();
        while let Some(ev) = term.try_read_event() {
            if ev.is_quit() {
                break 'outer;
            }

            if start.elapsed() >= max_budget {
                break;
            }

            if let crate::Event::Resize(size) = ev {
                last_resize = Some(size);
                continue;
            }

            surface.update(&ev);
            state.event(&ev);
            should_render = true;
        }

        if let Some(size) = last_resize {
            let ev = crate::Event::Resize(size);
            surface.update(&ev);
            state.event(&ev);
            should_render = true;
        }

        let now = Instant::now();
        let dt = prev.elapsed();
        state.update(dt.as_secs_f32());
        state.build(surface.rect(), |ui| app(ui));

        if should_render || dt >= target {
            state.render(&mut surface);
            surface.render(&mut term.writer())?;
            prev = now;
        }

        let elapsed = prev.elapsed();
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
    }

    Ok(())
}
