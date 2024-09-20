#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
use std::time::{Duration, Instant};

use too_math::Vec2;
use too_renderer::TermRenderer;
use too_shapes::Text;

mod fps;
use fps::Fps;

pub use too_events::{Event, Key, Keybind, Modifiers, MouseButton};
pub use too_renderer::{
    Attribute, Color, Command, CroppedSurface, Gradient, Pixel, Rgba, Shape, Surface,
};

pub use too_events::EventReader;
pub use too_renderer::Backend;

#[doc(inline)]
pub use too_math as math;

#[doc(inline)]
pub use too_shapes as shapes;

pub trait App {
    fn new(size: Vec2) -> Self
    where
        Self: Sized;

    fn event(&mut self, event: Event, ctx: Context<'_>, size: Vec2) {
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

    fn render(&mut self, surface: &mut Surface);
}

pub struct Context<'a> {
    show_fps: &'a mut bool,
    backend: &'a mut dyn Backend,
}

impl<'a> Context<'a> {
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

pub fn run<A: App + 'static>(mut term: impl Backend + EventReader) -> std::io::Result<()> {
    let mut surface = Surface::new(term.size());
    let mut app = A::new(term.size());

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
            event_dur += start.elapsed()
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

        if term.is_in_alt_screen() {
            app.render(&mut surface);

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
