use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::{
    backend::{Backend, Event, EventReader},
    overlay::Overlay,
    renderer::{Surface, TermRenderer},
};

mod context;
pub use context::Context;

/// Configurable type to 'hook' into different steps of the run loop
pub struct Runner<T> {
    frame_ready: fn(&mut T, Context<'_>),
    min_ups: fn(&T) -> f32,
    max_ups: fn(&T) -> f32,
    init: fn(&mut T, Context<'_>),
    event: fn(&mut T, Event, Context<'_>),
    update: fn(&mut T, f32, Context<'_>),
    render: for<'c> fn(&mut T, &mut Surface, Context<'_>),
    post_render: for<'c> fn(&mut T, overlay: &mut Overlay, &mut Surface),
}

impl<T> Default for Runner<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Runner<T> {
    pub const fn new() -> Self {
        Self {
            frame_ready: |_, _| {},
            min_ups: |_| 10.0,
            max_ups: |_| 60.0,
            init: |_, _| {},
            event: |_, _, _| {},
            update: |_, _, _| {},
            render: |_, _, _| {},
            post_render: |_, _, _| {},
        }
    }

    /// After updating and before rendering, this is called
    pub const fn frame_ready(mut self, ready: fn(&mut T, Context<'_>)) -> Self {
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

    pub const fn init(mut self, init: fn(&mut T, Context<'_>)) -> Self {
        self.init = init;
        self
    }

    pub const fn event(mut self, event: fn(&mut T, Event, Context<'_>)) -> Self {
        self.event = event;
        self
    }

    pub const fn update(mut self, update: fn(&mut T, f32, Context<'_>)) -> Self {
        self.update = update;
        self
    }

    pub const fn render(mut self, render: for<'c> fn(&mut T, &mut Surface, Context<'_>)) -> Self {
        self.render = render;
        self
    }

    pub const fn post_render(
        mut self,
        post_render: for<'c> fn(&mut T, &mut Overlay, &mut Surface),
    ) -> Self {
        self.post_render = post_render;
        self
    }

    pub fn run(self, mut state: T, mut term: impl Backend + EventReader) -> std::io::Result<()> {
        let mut overlay = Overlay::default();
        let mut commands = VecDeque::new();

        let mut surface = Surface::new(term.size());
        (self.init)(
            &mut state,
            Context {
                overlay: &mut overlay,
                commands: &mut commands,
                size: surface.rect().size(),
            },
        );

        let mut target_ups = (self.max_ups)(&state);
        let mut base_target = Duration::from_secs_f32(1.0 / target_ups);
        let mut prev = Instant::now();

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
                    overlay: &mut overlay,
                    commands: &mut commands,
                    size: surface.rect().size(),
                };
                (self.event)(&mut state, ev, ctx);
                event_dur += start.elapsed();

                // only spend up to half of the budget on reading events
                if event_dur >= base_target / 2 {
                    break;
                }
            }

            for cmd in commands.drain(..) {
                term.command(cmd);
            }

            let mut accum = frame_start.duration_since(prev).as_secs_f32();
            let target_dur = base_target.as_secs_f32();
            while accum >= target_dur {
                let start = Instant::now();
                let ctx = Context {
                    overlay: &mut overlay,
                    commands: &mut commands,
                    size: surface.rect().size(),
                };
                (self.update)(&mut state, target_dur, ctx);
                let update_dur = start.elapsed().as_secs_f32();
                accum -= target_dur;

                target_ups = if update_dur > target_dur {
                    (target_ups * 0.9).max((self.min_ups)(&state))
                } else {
                    (target_ups * 10.05).min((self.max_ups)(&state))
                }
            }

            let ctx = Context {
                overlay: &mut overlay,
                commands: &mut commands,
                size: surface.rect().size(),
            };
            (self.frame_ready)(&mut state, ctx);

            if term.should_draw() {
                let ctx = Context {
                    overlay: &mut overlay,
                    commands: &mut commands,
                    size: surface.rect().size(),
                };
                (self.render)(&mut state, &mut surface, ctx);
                (self.post_render)(&mut state, &mut overlay, &mut surface);
                surface.render(&mut TermRenderer::new(&mut term))?;
            }

            let total = frame_start.elapsed() - event_dur;
            if let Some(sleep) = base_target
                .checked_sub(total)
                .filter(|&d| d > Duration::ZERO)
            {
                std::thread::sleep(sleep);
            }

            overlay
                .fps
                .push(frame_start.duration_since(prev).as_secs_f32());

            prev = frame_start;
            base_target = Duration::from_secs_f32(1.0 / target_ups)
        }
    }
}
