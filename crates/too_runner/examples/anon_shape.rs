use too_crossterm::{Config, Term};
use too_runner::{pixel::Pixel, shapes::anonymous, App, AppRunner};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    Demo.run(term)
}

struct Demo;
impl App for Demo {
    fn render(&mut self, surface: &mut too_runner::SurfaceMut) {
        surface.draw(anonymous(|size| {
            move |pos| match () {
                _ if pos.x & 1 == 1 || pos.y & 1 == 1 => None,
                _ if pos.x < size.x / 2 => Some(Pixel::new(' ').bg("#F00")),
                _ if pos.y < size.y / 2 => Some(Pixel::new(' ').bg("#0F0")),
                _ => Some(Pixel::new(' ').bg("#00F")),
            }
        }));
    }
}
