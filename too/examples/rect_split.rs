use too_crossterm::*;

use too::{shapes::Fill, App, AppRunner, Context, Event, SurfaceMut};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    Test { t: 0.5, horz: true }.run(term)
}

struct Test {
    t: f32,
    horz: bool,
}

impl App for Test {
    fn event(&mut self, event: Event, _ctx: Context<'_>) {
        if event.is_keybind_pressed('d') {
            self.t += 0.1;
            self.t = self.t.clamp(0.0, 1.0);
        }
        if event.is_keybind_pressed('a') {
            self.t -= 0.1;
            self.t = self.t.clamp(0.0, 1.0);
        }
        if event.is_keybind_pressed(' ') {
            self.horz = !self.horz
        }
    }

    fn render(&mut self, mut surface: SurfaceMut, _ctx: Context) {
        let rect = surface.rect();

        let (main, cross) = match self.horz {
            true => rect.split_horizontal(1, self.t),
            false => rect.split_vertical(1, self.t),
        };

        surface.crop(main).draw(Fill::new("#F00"));
        surface.crop(cross).draw(Fill::new("#00F"));
    }
}
