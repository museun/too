use too_crossterm::{Config, Term};
use too_events::Event;
use too_immediate::{App, AppRunner as _};
use too_layout::{Anchor2, Axis};
use too_math::Rect;
use too_renderer::{Rgba, SurfaceMut};
use too_runner::Context;
use too_shapes::Fill;

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    Demo {
        axis: Axis::Horizontal,
        anchor: Anchor2::RIGHT_BOTTOM,
        fg: 0.0,
    }
    .run(term)
}

struct Demo {
    axis: Axis,
    anchor: Anchor2,
    fg: f32,
}

impl App for Demo {
    fn update(&mut self, dt: f32, mut ctx: Context<'_>) {
        self.fg += 1.0 * dt / 5.0_f32;
        ctx.overlay().fps.fg = Rgba::sine(self.fg);

        ctx.overlay()
            .debug
            .push(format!("fg phase: {:.2?}", self.fg));
    }

    fn event(&mut self, event: Event, mut ctx: Context<'_>) {
        if event.is_keybind_pressed('t') {
            ctx.overlay().fps.anchor = self.anchor;
            ctx.overlay().fps.axis = self.axis;
            ctx.toggle_fps();
        }

        if event.is_keybind_pressed('d') {
            ctx.overlay().debug.toggle();
        }
    }

    fn render(&mut self, mut surface: SurfaceMut, mut ctx: Context<'_>) {
        ctx.overlay()
            .debug
            .push(format!("surface size: {:?}", surface.rect().size()));

        surface
            .crop(Rect::from_center_size(
                surface.rect().center(),
                surface.rect().size() / 3,
            ))
            .draw(Fill::new("#555"));
    }
}
