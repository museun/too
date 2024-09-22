use std::time::{Duration, Instant};

use too_events::Event;
use too_math::Vec2;
use too_renderer::TermRenderer;
use too_shapes::Text;

mod fps;
use fps::Fps;

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
    pub use too_layout::{Anchor, Anchor2, Axis, LinearAllocator, LinearLayout};
}

#[doc(inline)]
pub use too_math as math;

/// Shapes are drawable primitives for a Surface
pub mod shapes {
    pub use too_renderer::Shape;
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
/// ```rust,no_run
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
/// fn main() -> std::io::Result<()> {
///     let backend = get_backend()?;
///     Demo::new(1234).run(backend)
/// }
/// ```
pub trait AppRunner: App + Sealed + Sized {
    /// Run the [`App`] with the provided [`Backend`] and [`EventReader`]
    fn run(self, term: impl Backend + EventReader) -> std::io::Result<()> {
        run_app(self, term)
    }
}

#[doc(hidden)]
pub trait Sealed {}

impl<T> Sealed for T {}
impl<T: App + Sealed> AppRunner for T {}

fn run_app(mut app: impl App, mut term: impl Backend + EventReader) -> std::io::Result<()> {
    let mut surface = too_renderer::Surface::new(term.size());
    app.initial_size(surface.rect().size());

    let mut target_ups = app.max_ups();
    let mut base_target = Duration::from_secs_f32(1.0 / target_ups);
    let mut fps = <Fps<32>>::new();
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
            app.event(ev, ctx, surface.rect().size());
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
            app.update(target_dur, surface.rect().size());
            let update_dur = start.elapsed().as_secs_f32();
            accum -= target_dur;

            target_ups = if update_dur > target_dur {
                (target_ups * 0.9).max(app.min_ups())
            } else {
                (target_ups * 10.05).min(app.max_ups())
            }
        }

        // and if we have any remaining from the time slice, do it again
        if accum > 0.0 {
            let start = Instant::now();
            app.update(accum, surface.rect().size());
            let update_dur = start.elapsed().as_secs_f32();
            target_ups = if update_dur > target_dur {
                (target_ups * 0.9).max(app.min_ups())
            } else {
                (target_ups * 10.05).min(app.max_ups())
            }
        }

        if term.should_draw() {
            app.render(&mut surface.crop(surface.rect()));

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

        let total = frame_start.elapsed() + event_dur;
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
