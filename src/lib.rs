#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
use compact_str::{CompactString, ToCompactString};
use layout::Anchor2;
use std::ops::Deref;
use view::{DebugMode, Palette};

pub mod backend;
#[doc(inline)]
pub use backend::{Key, Keybind, Modifiers, MouseButton};

pub mod math;

pub mod renderer;

#[doc(inline)]
pub use renderer::{Attribute, Border, Cell, Color, Gradient, Grapheme, Pixel, Rgba, Underline};

pub mod layout;

pub mod animation;
pub use animation::Animations;

pub mod view;
pub mod views;

mod rasterizer;
pub use rasterizer::{Rasterizer, Shape, TextShape};

// TODO get rid of this
use crate::math::Size;
#[inline(always)]
#[deprecated(note = "don't use this, use Text when its implemented")]
pub fn measure_text(data: &str) -> Size {
    use unicode_width::UnicodeWidthStr as _;
    Size::new(data.width() as f32, 1.0)
}

#[cfg(feature = "terminal")]
pub mod term;

pub mod hasher;

#[doc(hidden)]
pub use compact_str::format_compact as __dont_use_this_because_semver;
// basically just https://github.com/ParkMyCar/compact_str/blob/193d13eaa5a92b3c39c2f7289dc44c95f37c80d1/compact_str/src/macros.rs#L28
// but semver-safe
/// Like [`std::format!`] but for a [`Str`]
#[macro_export]
macro_rules! format_str {
    ($($arg:tt)*) => {
        $crate::Str::from($crate::__dont_use_this_because_semver!($($arg)*))
    }
}

/// A semver wrapper around a [`CompactString`](https://docs.rs/compact_str/0.8.0/compact_str/index.html)
///
/// You would normally not need to name this type, anything that implements [`ToCompactString`](https://docs.rs/compact_str/0.8.0/compact_str/trait.ToCompactString.html) can be turned into this type.
///
/// You can use [`format_str!`] like [`std::format!`] to make this type
/// - or `Str::from(&str)`
/// - or `Str::from(String)`
/// - or `Str::from(usize)`
/// - etc
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Str(CompactString);

impl Str {
    pub const fn new(str: &'static str) -> Self {
        Self(CompactString::const_new(str))
    }
}

impl AsRef<str> for Str {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Deref for Str {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl<T> From<T> for Str
where
    T: ToCompactString,
{
    fn from(value: T) -> Self {
        Self(value.to_compact_string())
    }
}

pub struct Config {
    pub palette: Palette,
    pub debug: DebugMode,
    pub debug_anchor: Anchor2,
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
            debug_anchor: Anchor2::RIGHT_TOP,
            animation: Animations::default(),
            fps: 60.0,
            ctrl_c_quits: true,
            ctrl_z_switches: false,
            hook_panics: false,
        }
    }
}

#[cfg(feature = "terminal")]
pub fn run<R: 'static>(app: impl FnMut(&crate::view::Ui) -> R) -> std::io::Result<()> {
    application(Config::default(), app)
}

#[cfg(feature = "terminal")]
pub fn application<R: 'static>(
    config: Config,
    mut app: impl FnMut(&crate::view::Ui) -> R,
) -> std::io::Result<()> {
    use std::time::{Duration, Instant};

    use crate::{
        backend::{Backend, Event, EventReader},
        renderer::Surface,
        term::{Config as TermConfig, Term},
        view::{CroppedSurface, State},
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
    state.set_debug_anchor(config.debug_anchor);

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
