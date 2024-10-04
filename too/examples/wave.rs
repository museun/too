use std::f32::consts::TAU;

use too::{
    math::{lerp, pos2},
    App, AppRunner, Rgba,
};
use too_crossterm::{Config, Term};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    (Wave {
        theta: 0.0,
        fg: Rgba::hex("#7c68ee"),
        bg: Rgba::hex("#a52a2a"),
    })
    .run(term)
}

struct Wave {
    theta: f32,
    fg: Rgba,
    bg: Rgba,
}

impl Wave {
    fn compute(&self, width: i32) -> impl Iterator<Item = Rgba> + Clone + '_ {
        let mut time = 0.0f32;
        std::iter::repeat(()).map(move |()| {
            let value = 0.5 + 0.5 * (TAU * time + 5.0).sin();
            time += 1.0 / width as f32;
            self.fg.blend_linear(self.bg, value)
        })
    }
}

impl App for Wave {
    fn update(&mut self, dt: f32, _ctx: too::Context<'_>) {
        self.theta += dt;
        self.theta %= 1.0;
    }

    fn render(&mut self, surface: &mut too::Surface, _ctx: too::Context<'_>) {
        let rect = surface.rect();
        let size = rect.size();
        let w = size.x;
        let t = lerp(0.0, w as f32, self.theta);

        for (i, n) in self
            .compute(w)
            .skip(t as usize)
            .take(w as usize)
            .enumerate()
        {
            let x = rect.left() + i as i32;
            for y in rect.top()..rect.bottom() {
                surface.set(pos2(x, y), n);
            }
        }
    }
}
