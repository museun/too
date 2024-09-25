use std::time::{Duration, Instant};

use too_events::Event;
use too_math::Vec2;
use too_renderer::TermRenderer;
use too_shapes::Text;

mod ema_window;
pub use ema_window::EmaWindow;

pub use too_renderer::{Backend, Command, SurfaceMut};

/// Color types
pub mod color {
    pub use too_renderer::{Gradient, Rgba};
}

/// Pixels are what a Surface is consists of
pub mod pixel {
    pub use too_renderer::{Attribute, Color, Pixel};
}

/// Events sent to your application
pub mod events {
    pub use too_events::{Event, Key, Keybind, Modifiers, MouseButton};
}

pub use too_events::EventReader;

/// Layout helpers
pub mod layout {
    pub use too_layout::{
        Align, Align2, Anchor, Anchor2, Axis, Constraints, LinearAllocator, LinearLayout, Size,
    };
}

#[doc(inline)]
pub use too_math as math;

/// Shapes are drawable primitives for a Surface
pub mod shapes {
    pub use too_renderer::{anonymous, anonymous_ctx, Shape};
    pub use too_shapes::*;
}

/// Trait for defining an application to run
pub trait App {
    /// The initial surface size that you can use to compute some internal state
    fn initial_size(&mut self, size: Vec2) {
        _ = size
    }

    /// An [`Event`] was sent from the backend
    ///
    /// This provides a [`Context`] to the backend, and the current surface `size`
    fn event(&mut self, event: Event, ctx: Context<'_, impl Backend>, size: Vec2) {
        _ = event;
        _ = ctx;
        _ = size;
    }

    /// Update allows you to interpolate state
    ///
    /// `dt` is the delta-time that can be used for interpolation
    /// `size` is the current surface size
    fn update(&mut self, dt: f32, size: Vec2) {
        _ = dt;
        _ = size;
    }

    /// Min UPS is the minimum UPS the runner should perform
    ///
    /// The default is 10 updates/s
    fn min_ups(&self) -> f32 {
        10.0
    }

    /// Max UPS is the maximum UPS the runner should perform
    ///
    /// The default is 60 updates/s
    fn max_ups(&self) -> f32 {
        60.0
    }

    /// Render your application
    ///
    /// This provides you with a [`SurfaceMut`] that allows you to draw onto
    ///
    /// The draw order are back-to-front. Later draw calls will be drawn over earlier calls
    fn render(&mut self, surface: &mut SurfaceMut);
}

/// Context to the [`Backend`] for use during [`App::event`]
///
/// This allows you to communicate with the backend when it sends an event
pub struct Context<'a, B: Backend> {
    show_fps: &'a mut bool,
    backend: &'a mut B,
}

impl<'a, B: Backend> Context<'a, B> {
    /// Should we show the FPS overlay?
    pub fn show_fps(&mut self, show_fps: bool) {
        *self.show_fps = show_fps
    }

    /// Toggle the FPS overlay
    pub fn toggle_fps(&mut self) {
        *self.show_fps = !*self.show_fps
    }

    /// Send a [`Command`] to the backend
    ///
    /// Commands are things like:
    /// * Quit
    /// * Set the title
    pub fn command(&mut self, cmd: Command) {
        self.backend.command(cmd);
    }
}

/// A trait to run your application
///
/// It is implemented for all types that implement [`App`].
///
/// # Example:
/// ```rust
/// use too_runner::{AppRunner as _, SurfaceMut};
///
/// struct Demo {
///     state: i32
/// }
///
/// impl Demo {
///     fn new(state: i32) -> Self {
///         Self { state }
///     }
/// }
///
/// impl too_runner::App for Demo {
///     fn render(&mut self, surface: &mut SurfaceMut) {}
/// }
///
/// # fn get_backend() -> std::io::Result<too_runner::dummy::Dummy> { Ok(too_runner::dummy::Dummy) }
/// fn main() -> std::io::Result<()> {
///     let backend = get_backend()?;
///     Demo::new(1234).run(backend)
/// }
/// ```
pub trait AppRunner: App + Sealed + Sized {
    /// Run the [`App`] with the provided [`Backend`] and [`EventReader`]
    fn run(self, term: impl Backend + EventReader) -> std::io::Result<()> {
        Runner::new()
            .min_ups(App::min_ups)
            .max_ups(App::max_ups)
            .init(App::initial_size)
            .event(App::event)
            .update(App::update)
            .render(App::render)
            .run(self, term)
    }
}

#[doc(hidden)]
pub trait Sealed {}

impl<T> Sealed for T {}
impl<T: App + Sealed> AppRunner for T {}

/// Configurable type to 'hook' into different steps of the run loop
pub struct Runner<T, B: Backend> {
    frame_ready: fn(&mut T),
    min_ups: fn(&T) -> f32,
    max_ups: fn(&T) -> f32,
    init: fn(&mut T, Vec2),
    event: fn(&mut T, Event, Context<'_, B>, Vec2),
    update: fn(&mut T, f32, Vec2),
    render: fn(&mut T, &mut SurfaceMut<'_>),
}

impl<T, B: Backend + EventReader> Default for Runner<T, B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, B: Backend + EventReader> Runner<T, B> {
    pub const fn new() -> Self {
        Self {
            frame_ready: |_| {},
            min_ups: |_| 10.0,
            max_ups: |_| 60.0,
            init: |_, _| {},
            event: |_, _, _, _| {},
            update: |_, _, _| {},
            render: |_, _| {},
        }
    }

    /// After updating and before rendering, this is called
    pub const fn frame_ready(mut self, ready: fn(&mut T)) -> Self {
        self.frame_ready = ready;
        self
    }

    pub const fn min_ups(mut self, min_ups: fn(&T) -> f32) -> Self {
        self.min_ups = min_ups;
        self
    }

    pub const fn max_ups(mut self, max_ups: fn(&T) -> f32) -> Self {
        self.max_ups = max_ups;
        self
    }

    pub const fn init(mut self, init: fn(&mut T, Vec2)) -> Self {
        self.init = init;
        self
    }

    pub const fn event(mut self, event: fn(&mut T, Event, Context<'_, B>, Vec2)) -> Self {
        self.event = event;
        self
    }

    pub const fn update(mut self, update: fn(&mut T, f32, Vec2)) -> Self {
        self.update = update;
        self
    }

    pub const fn render(mut self, render: fn(&mut T, &mut SurfaceMut<'_>)) -> Self {
        self.render = render;
        self
    }

    pub fn run(self, mut state: T, mut term: B) -> std::io::Result<()> {
        let mut surface = too_renderer::Surface::new(term.size());
        (self.init)(&mut state, surface.rect().size());

        let mut target_ups = (self.max_ups)(&state);
        let mut base_target = Duration::from_secs_f32(1.0 / target_ups);
        let mut fps = <EmaWindow<32>>::new();
        let mut prev = Instant::now();

        let mut show_fps = false;

        loop {
            let frame_start = Instant::now();

            let mut event_dur = Duration::ZERO;
            while let Some(ev) = term.try_read_event() {
                if ev.is_quit() {
                    return Ok(());
                }

                let start = Instant::now();
                surface.update(&ev);
                let ctx = Context {
                    show_fps: &mut show_fps,
                    backend: &mut term,
                };
                (self.event)(&mut state, ev, ctx, surface.rect().size());
                event_dur += start.elapsed();

                // only spend up to half of the budget on reading events
                if event_dur >= base_target / 2 {
                    break;
                }
            }

            let mut accum = frame_start.duration_since(prev).as_secs_f32();
            let target_dur = base_target.as_secs_f32();
            while accum >= target_dur {
                let start = Instant::now();
                (self.update)(&mut state, target_dur, surface.rect().size());
                let update_dur = start.elapsed().as_secs_f32();
                accum -= target_dur;

                target_ups = if update_dur > target_dur {
                    (target_ups * 0.9).max((self.min_ups)(&state))
                } else {
                    (target_ups * 10.05).min((self.max_ups)(&state))
                }
            }

            // and if we have any remaining from the time slice, do it again
            if accum > 0.0 {
                let start = Instant::now();
                (self.update)(&mut state, accum, surface.rect().size());
                let update_dur = start.elapsed().as_secs_f32();
                target_ups = if update_dur > target_dur {
                    (target_ups * 0.9).max((self.min_ups)(&state))
                } else {
                    (target_ups * 10.05).min((self.max_ups)(&state))
                }
            }

            (self.frame_ready)(&mut state);

            if term.should_draw() {
                (self.render)(&mut state, &mut surface.crop(surface.rect()));

                if show_fps {
                    let frame_stats = fps.get();
                    let label = format!(
                        "min: {:.2}, max: {:.2}, avg: {:.2}",
                        frame_stats.min, frame_stats.max, frame_stats.avg,
                    );
                    surface.draw(Text::new(label).fg("#F00").bg("#000"));
                }
                surface.render(&mut TermRenderer::new(&mut term))?;
            }

            let total = frame_start.elapsed() - event_dur;
            if let Some(sleep) = base_target
                .checked_sub(total)
                .filter(|&d| d > Duration::ZERO)
            {
                std::thread::sleep(sleep);
            }

            fps.push(frame_start.duration_since(prev).as_secs_f32());

            prev = frame_start;
            base_target = Duration::from_secs_f32(1.0 / target_ups)
        }
    }
}

// Hide this from the docs
// #[cfg(doctests)] doesn't work as expected here
#[doc(hidden)]
pub mod dummy;
