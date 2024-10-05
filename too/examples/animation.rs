use std::time::{Duration, Instant};

use too::{
    animation::{easing::*, Animation},
    math::{lerp, pos2, vec2, Rect},
    App, AppRunner, Rgba,
};
use too_crossterm::{Config, Term};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    Demo::new().run(term)
}

struct Demo {
    start: Instant,
}

impl Demo {
    fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl App for Demo {
    fn initial_size(&mut self, mut ctx: too::Context<'_>) {
        ctx.overlay().debug.show = true;

        let initial_x = ctx.size().x as f32 * 0.5;
        ctx.animations_mut().add(
            "xt",
            Animation::new()
                .repeat(true)
                .round_trip(true)
                .with_time(|d| linear(d), Duration::from_secs(2))
                .with(|d| elastic_in_out(d))
                .schedule(Duration::from_secs(10))
                .unwrap(),
            initial_x,
        );
    }

    fn update(&mut self, _dt: f32, mut ctx: too::Context<'_>) {
        let x = *ctx.animations().value("xt").value;

        ctx.overlay().debug.push(format!("x: {x:.2?}"));

        ctx.overlay()
            .debug
            .push(format!("{:.2?}", self.start.elapsed()));
    }

    fn event(&mut self, event: too::Event, mut ctx: too::Context<'_>) {
        if event.is_keybind_pressed('d') {
            ctx.overlay().debug.toggle();
        }

        if event.is_keybind_pressed('r') {
            ctx.animations_mut().value_mut("xt").animation.reset();
        }
    }

    fn render(&mut self, surface: &mut too::Surface, ctx: too::Context<'_>) {
        let x = *ctx.animations().value("xt").value;

        let rect = surface.rect();
        let size = rect.size() / 4;

        let offset = vec2(lerp(0.0, (rect.width() + 1 - size.x) as f32, x) as i32, 0);
        let rect = Rect::from_min_size(pos2(0, rect.center().y), size).translate(offset);

        surface.fill(surface.rect(), Rgba::hex("#000"));
        surface.fill(
            rect,
            Rgba::hex("#F0F").to_transparent(x.clamp(0.3, 1.0) * 100.0),
        );
    }
}
