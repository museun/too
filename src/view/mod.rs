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
pub use style::{Elements, Palette, StyleKind};

mod internal_views;

mod adhoc;
pub use adhoc::Adhoc;

mod builder;
pub use builder::{Builder, View, ViewExt};

use crate::Animations;

mod erased;
use erased::Erased;

slotmap::new_key_type! {
    pub struct ViewId;
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
            let mut rasterizer = CroppedSurface {
                clip_rect: surface.rect(),
                surface: &mut surface,
            };
            state.render(&mut rasterizer);
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
