use std::time::{Duration, Instant};

use too_events::Event;
use too_math::Vec2;
use too_renderer::TermRenderer;
use too_shapes::Text;

mod fps;
use fps::Fps;

pub use too_renderer::{Backend, Command, SurfaceMut};

pub mod color {
    pub use too_renderer::{Gradient, Rgba};
}

pub mod pixel {
    pub use too_renderer::{Attribute, Color, Pixel};
}

pub mod events {
    pub use too_events::{Event, Key, Keybind, Modifiers, MouseButton, MouseState};
}

pub use too_events::EventReader;

#[doc(inline)]
pub use too_math as math;

pub mod shapes {
    pub use too_renderer::Shape;
    pub use too_shapes::*;
}

pub trait App {
    fn event(&mut self, event: Event, ctx: Context<'_, impl Backend>, size: Vec2) {
        _ = event;
        _ = ctx;
        _ = size;
    }

    fn update(&mut self, dt: f32, size: Vec2) {
        _ = dt;
        _ = size;
    }

    fn min_ups(&self) -> f32 {
        10.0
    }

    fn max_ups(&self) -> f32 {
        60.0
    }

    fn render(&mut self, surface: &mut SurfaceMut);
}

pub struct Context<'a, B: Backend> {
    show_fps: &'a mut bool,
    backend: &'a mut B,
}

impl<'a, B: Backend> Context<'a, B> {
    pub fn show_fps(&mut self, show_fps: bool) {
        *self.show_fps = show_fps
    }

    pub fn toggle_fps(&mut self) {
        *self.show_fps = !*self.show_fps
    }

    pub fn command(&mut self, cmd: Command) {
        self.backend.command(cmd);
    }
}

pub fn run<A: App>(
    app: impl FnOnce(Vec2) -> A,
    mut term: impl Backend + EventReader,
) -> std::io::Result<()> {
    let mut surface = too_renderer::Surface::new(term.size());
    let mut app = app(surface.rect().size());

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
