use too::{
    anonymous, anonymous_ctx, pos2, App, AppRunner as _, Context, Pixel, Rect, Rgba, SurfaceMut,
};
use too_crossterm::{Config, Term};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    Demo { t: 0.0 }.run(term)
}

struct Demo {
    t: f32,
}

impl App for Demo {
    fn update(&mut self, dt: f32, _ctx: Context<'_>) {
        self.t += 1.0 * dt * 1.0 / 2.0
    }

    fn render(&mut self, mut surface: SurfaceMut<'_>, _ctx: Context<'_>) {
        surface
            .draw(anonymous_ctx(&self, |size| {
                move |this, pos| match () {
                    _ if pos.x & 1 == 1 || pos.y & 1 == 1 => None,
                    _ if pos.x < size.x / 2 => Some(Pixel::new('░').bg(Rgba::sine(this.t))),
                    _ if pos.y < size.y / 2 => Some(Pixel::new('░').bg(Rgba::sine(this.t + 1.0))),
                    _ => Some(Pixel::new('░').bg(Rgba::sine(this.t + 2.0))),
                }
            }))
            .draw(anonymous(|size| {
                let rect = Rect::from_center_size(pos2(size.x / 2, size.y / 2), size / 3);
                move |pos| rect.contains(pos).then(|| Pixel::new(' ').bg("#333A"))
            }));
    }
}
