use too_crossterm::{Config, Term};

use too::{
    math::{rect, vec2, Pos2, Rect},
    App, AppRunner as _, Command, Context, Event, Justification, Rgba, Surface, Text,
};

fn main() -> std::io::Result<()> {
    let term_config = Config::default().hook_panics(true).ctrl_z_switches(true);
    let term = Term::setup(term_config)?;
    Hello::new().run(term)
}

#[derive(Copy, Clone, Default)]
enum Grabbed {
    Held,
    Hover,
    Grabbed,
    #[default]
    None,
}

struct Hello {
    value: f32,
    up: bool,
    alpha: f32,
    grabbed: Grabbed,

    pos: Pos2,
    rect: Rect,
}

impl Hello {
    const fn new() -> Self {
        Self {
            value: 0.0,
            up: true,
            alpha: 0.5,
            grabbed: Grabbed::None,
            pos: Pos2::ZERO,
            rect: Rect::from_min_size(Pos2::ZERO, vec2(20, 6)),
        }
    }
}

impl App for Hello {
    fn event(&mut self, event: Event, mut ctx: Context<'_>) {
        if event.is_keybind_pressed('q') {
            ctx.command(Command::request_quit());
        }

        if event.is_keybind_pressed('t') {
            ctx.toggle_fps();
        }

        if let Some(pos) = event.mouse_pos() {
            self.grabbed = if self.rect.contains(pos) {
                Grabbed::Hover
            } else {
                Grabbed::None
            }
        }

        if let Event::MouseHeld { pos, .. } = event {
            if self.rect.contains(pos) {
                self.grabbed = Grabbed::Held
            }
        }

        if let Event::MouseDragStart { pos, .. } = event {
            if self.rect.contains(pos) {
                self.grabbed = Grabbed::Grabbed;
            }
        }

        if let Event::MouseDragHeld { pos, delta, .. } = event {
            if self.rect.contains(pos) {
                self.grabbed = Grabbed::Grabbed;
                self.rect = self.rect.translate(delta);
                self.rect = rect(ctx.size()).clamp_rect(self.rect);
            }
        }

        if let Event::MouseDragRelease { pos, .. } = event {
            if self.rect.contains(pos) {
                self.grabbed = Grabbed::Hover
            }
        }

        if let Event::MouseScroll { pos, delta, .. } = event {
            if self.rect.contains(pos) {
                self.alpha += -delta.y as f32 * 0.1;
                self.alpha = self.alpha.clamp(0.0, 1.0);
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

    fn update(&mut self, dt: f32, _ctx: Context<'_>) {
        let duration = 5.0f32;
        self.value += (self.up as u8 as f32 * 2.0 - 1.0) * duration.recip() * dt;
        self.value = self.value.clamp(0.0, 1.0);
        self.up = self.up ^ (self.value >= 1.0) ^ (self.value <= 0.0)
    }

    fn render(&mut self, surface: &mut Surface, _ctx: Context<'_>) {
        let rect = surface.rect();

        surface.fill(
            Rect::from_center_size(rect.center(), rect.size() / 3),
            Rgba::sine(self.value),
        );

        let view_color = match self.grabbed {
            Grabbed::Held => "#173",
            Grabbed::Hover => "#127",
            Grabbed::Grabbed => "#723",
            Grabbed::None => "#123",
        };

        surface
            .fill(
                self.rect,
                Rgba::hex(view_color).to_transparent(self.alpha * 100.0),
            )
            .text(
                self.rect,
                Text::new(format!("{},{}", self.pos.x, self.pos.y))
                    .fg("#FFF")
                    .main(Justification::Center)
                    .cross(Justification::Center),
            )
            .text(
                rect,
                Text::new(format!("{:?}", self.rect))
                    .main(Justification::End)
                    .cross(Justification::Start),
            );
    }
}
