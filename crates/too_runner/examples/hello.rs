use too_crossterm::{Config, Term};

use too_runner::{
    events::Event,
    math::{rect, vec2, Align2, Pos2, Rect, Vec2},
    shapes::{Fill, Text},
    App, Backend, Command, Context, Rgba, Surface,
};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default().hook_panics(true).ctrl_z_switches(true))?;
    too_runner::run(|_| Hello::new(), term)
}

struct Hello {
    value: f32,
    up: bool,

    pos: Pos2,
    rect: Rect,
}

impl Hello {
    const fn new() -> Self {
        Self {
            value: 0.0,
            up: true,
            pos: Pos2::ZERO,
            rect: Rect::from_min_size(Pos2::ZERO, vec2(20, 6)),
        }
    }
}

impl App for Hello {
    fn event(&mut self, event: Event, mut ctx: Context<'_, impl Backend>, size: Vec2) {
        if event.is_keybind_pressed('q') {
            ctx.command(Command::request_quit());
        }

        if event.is_keybind_pressed('t') {
            ctx.toggle_fps();
        }

        if let Event::MouseDragHeld { pos, delta, .. } = event {
            if self.rect.contains(pos) {
                self.rect = self.rect.translate(delta);
                self.rect = rect(size).clamp_rect(self.rect);
            }
        }

        if let Event::Resize(new_size) = event {
            self.rect = Rect::from_min_size(self.rect.min, vec2(20, 6));
            self.rect = rect(new_size).clamp_rect(self.rect);
        }

        if let Some(pos) = event.mouse_pos() {
            self.pos = pos
        }
    }

    fn update(&mut self, dt: f32, _size: Vec2) {
        let duration = 5.0f32;
        self.value += (self.up as u8 as f32 * 2.0 - 1.0) * duration.recip() * dt;
        self.value = self.value.clamp(0.0, 1.0);
        self.up = self.up ^ (self.value >= 1.0) ^ (self.value <= 0.0)
    }

    fn render(&mut self, surface: &mut Surface) {
        let rect = surface.rect();
        surface
            .crop(Rect::from_center_size(rect.center(), rect.size() / 3))
            .draw(Fill::new(Rgba::sine(self.value)));

        surface
            .crop(self.rect)
            .draw(Fill::new(Rgba::from_u16(0x333A)))
            .draw(
                Text::new(format!("{},{}", self.pos.x, self.pos.y))
                    .fg("#FFF")
                    .align2(Align2::CENTER_CENTER),
            );

        surface.draw(Text::new(format!("{:?}", self.rect)).align2(Align2::RIGHT_TOP));
    }
}
