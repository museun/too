use too::{
    math::{pos2, rect, Pos2},
    App, AppRunner, Rgba,
};
use too_crossterm::{Config, Term};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    Vista::new().run(term)
}

struct Vista {
    pos: Pos2,
    flip: i32,
}

impl Vista {
    fn new() -> Self {
        Self {
            pos: Pos2::default(),
            flip: 1,
        }
    }
}

impl App for Vista {
    fn initial_size(&mut self, ctx: too::Context<'_>) {
        self.pos = rect(ctx.size()).center();
    }

    fn update(&mut self, _dt: f32, ctx: too::Context<'_>) {
        if self.pos.x >= ctx.size().x || self.pos.x <= 0 {
            self.flip = -self.flip
        }

        let h = ctx.size().y as f32;
        let top = 0 as f32;

        let x = (self.pos.x).saturating_add(self.flip);
        let sin = (self.pos.x as f32 / 10.0).sin();
        let y = h * sin / 4.0 + top + h / 2.0;
        self.pos = pos2(x, y as i32)
    }

    fn render(&mut self, surface: &mut too::Surface, _ctx: too::Context<'_>) {
        for y in 0..surface.rect().height() {
            for x in 0..surface.rect().width() {
                let dx = x as f32 - self.pos.x as f32;
                let dy = 2.0 * y as f32 - 2.0 * self.pos.y as f32;
                let distance = (dx * dx + dy * dy).sqrt();

                let fg = Rgba::new(
                    (255.0 * 10.0 / distance).clamp(1.0, 255.0) as u8,
                    255 - (255.0 * self.pos.y as f32 / surface.rect().height() as f32)
                        .clamp(1.0, 255.0) as u8,
                    (distance * 5.0).clamp(1.0, 255.0) as u8,
                    0xFF,
                );

                // surface.set(pos2(x, y), Pixel::new(' ').bg(fg));
                surface.set(pos2(x, y), fg);
            }
        }
    }
}
