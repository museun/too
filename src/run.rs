//! An event loop implementation to run ***too*** applications
//!
//! This is only enabled with the `terminal` feature is enabled
use crate::{
    animation::Animations,
    layout::Anchor2,
    view::{DebugMode, Palette},
};

/// Configuration for an [`application`]
///
/// ## Default configuration:
/// | Option | Value |
/// | --- | --- |
/// | [`palette`](Self::palette) | [`Palette::dark()`] |
/// | [`debug`](Self::debug) | [`DebugMode::PerFrame`] |
/// | [`debug_anchor`](Self::debug_anchor) | [`Anchor2::RIGHT_TOP`] |
/// | [`fps`](Self::fps) | `60.0` (e.g. 60 fps) |
/// | [`ctrl_c_quits`](Self::ctrl_c_quits) | `true` |
/// | [`ctrl_z_switches`](Self::ctrl_z_switches) | `false` |
/// | [`hook_panics`](Self::hook_panics) | `false` |

pub struct RunConfig {
    /// The palette to initially use
    ///
    /// Default: [`Palette::dark()`]
    pub palette: Palette,
    /// The initial [`DebugMode`] for the debug overlay
    ///
    /// Default: [`DebugMode::PerFrame`]
    pub debug: DebugMode,
    /// Where the debug overlay should be anchored
    ///
    /// Default: [`Anchor2::RIGHT_TOP`]
    pub debug_anchor: Anchor2,
    /// The animation manager
    pub animation: Animations,
    /// The framerate the application should run at
    ///
    /// Default: `60.0` (e.g. 60 fps)
    pub fps: f32,
    /// Should pressing Ctrl-C quit the application?
    ///
    /// Default: `true`
    pub ctrl_c_quits: bool,
    /// Should pressing Ctrl-Z switch to the non-displayed screen?
    ///
    /// Default: `false`
    pub ctrl_z_switches: bool,
    /// Should we attempt to hook panics to display after an application panics?
    ///
    /// Default: `false`
    pub hook_panics: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            palette: Palette::dark(),
            debug: DebugMode::PerFrame,
            debug_anchor: Anchor2::RIGHT_TOP,
            animation: Animations::default(),
            fps: 60.0,
            ctrl_c_quits: true,
            ctrl_z_switches: false,
            hook_panics: false,
        }
    }
}

// TODO more description on how the closure can be used
/// Run an application with the default [`RunConfig`]
///
/// This will block the current thread until the application exits.
#[cfg(feature = "terminal")]
pub fn run<R: 'static>(app: impl FnMut(&crate::view::Ui) -> R) -> std::io::Result<()> {
    application(RunConfig::default(), app)
}

/// Run an application with the provided [`RunConfig`]
///
/// This will block the current thread until the application exits.
#[cfg(feature = "terminal")]
pub fn application<R: 'static>(
    config: RunConfig,
    mut app: impl FnMut(&crate::view::Ui) -> R,
) -> std::io::Result<()> {
    use std::time::{Duration, Instant};

    use crate::{
        backend::{Backend, Event, EventReader},
        renderer::Surface,
        term::{Config as TermConfig, Term},
        view::{CroppedSurface, Debug, State},
    };

    let mut term = Term::setup(
        TermConfig::default()
            .hook_panics(config.hook_panics)
            .ctrl_c_quits(config.ctrl_c_quits)
            .ctrl_z_switches(config.ctrl_z_switches),
    )?;
    let mut surface = Surface::new(term.size());

    let mut state = State::new(config.palette, config.animation);
    Debug::set_debug_mode(config.debug);
    Debug::set_debug_anchor(config.debug_anchor);

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

            if let Event::Resize(size) = ev {
                last_resize = Some(size);
                continue;
            }

            surface.update(&ev);
            state.event(&ev);
            should_render = true;
        }

        if let Some(size) = last_resize {
            let ev = Event::Resize(size);
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
