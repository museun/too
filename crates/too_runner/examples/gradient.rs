use too_crossterm::{Config, Term};

use too_runner::{
    math::{lerp, pos2, Align2},
    shapes::Text,
    App, Context, Gradient, Keybind, Pixel, Shape,
};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default().hook_panics(true))?;
    too_runner::run(|_| Demo::new(), term)
}

struct Demo {
    gradients: [(&'static str, Gradient); 20],
    theta: f32,
    skew: f32,
    duration: f32,
    pos: usize,
    up: bool,
}

impl Demo {
    const fn new() -> Self {
        Self {
            pos: 0,
            theta: 0.0,
            skew: 1.0,
            duration: 5.0,
            up: true,
            gradients: [
                ("RAINBOW1", Gradient::RAINBOW1),
                ("RAINBOW2", Gradient::RAINBOW2),
                ("RAINBOW3", Gradient::RAINBOW3),
                ("RAINBOW4", Gradient::RAINBOW4),
                ("YELLOW_MAGENTA_CYAN", Gradient::YELLOW_MAGENTA_CYAN),
                ("ORANGE_BLUE", Gradient::ORANGE_BLUE),
                ("GREEN_MAGENTA", Gradient::GREEN_MAGENTA),
                ("GREEN_RED", Gradient::GREEN_RED),
                ("GREEN_CYAN", Gradient::GREEN_CYAN),
                ("YELLOW_RED", Gradient::YELLOW_RED),
                ("BLUE_CYAN", Gradient::BLUE_CYAN),
                ("RED_BLUE", Gradient::RED_BLUE),
                ("YELLOW_GREEN_BLUE", Gradient::YELLOW_GREEN_BLUE),
                ("BLUE_WHITE_RED", Gradient::BLUE_WHITE_RED),
                ("CYAN_MAGENTA", Gradient::CYAN_MAGENTA),
                ("YELLOW_PURPLE_MAGENTA", Gradient::YELLOW_PURPLE_MAGENTA),
                ("GREEN_BLUE_ORANGE", Gradient::GREEN_BLUE_ORANGE),
                ("ORANGE_MAGENTA_BLUE", Gradient::ORANGE_MAGENTA_BLUE),
                ("BLUE_MAGENTA_ORANGE", Gradient::BLUE_MAGENTA_ORANGE),
                ("MAGENTA_GREEN", Gradient::MAGENTA_GREEN),
            ],
        }
    }
}
impl App for Demo {
    fn event(&mut self, event: too_events::Event, mut ctx: Context<'_>, _size: too_math::Vec2) {
        const NEXT_GRADIENT: Keybind = Keybind::from_char('d');
        const PREV_GRADIENT: Keybind = Keybind::from_char('a');

        const SPEED_UP: Keybind = Keybind::from_char('w');
        const SPEED_DOWN: Keybind = Keybind::from_char('s');

        const SKEW_MORE: Keybind = Keybind::from_char('1');
        const SKEW_LESS: Keybind = Keybind::from_char('2');

        if event.is_keybind_pressed('t') {
            ctx.toggle_fps();
        }

        if event.is_keybind_pressed(SKEW_LESS) {
            self.skew += 0.1;
            self.skew = self.skew.clamp(0.1, 10.0);
        }
        if event.is_keybind_pressed(SKEW_MORE) {
            self.skew -= 0.1;
            self.skew = self.skew.clamp(0.1, 10.0);
        }

        if event.is_keybind_pressed(SPEED_UP) {
            self.duration += 1.0;
            self.duration = self.duration.clamp(1.0, 10.0);
        }
        if event.is_keybind_pressed(SPEED_DOWN) {
            self.duration -= 1.0;
            self.duration = self.duration.clamp(1.0, 10.0);
        }

        if event.is_keybind_pressed(NEXT_GRADIENT) {
            self.pos = (self.pos + 1) % self.gradients.len()
        }
        if event.is_keybind_pressed(PREV_GRADIENT) {
            self.pos = self.pos.checked_sub(1).unwrap_or(self.gradients.len() - 1)
        }
    }

    fn update(&mut self, dt: f32, _size: too_math::Vec2) {
        self.theta += (self.up as u8 as f32 * 2.0 - 1.0) * self.duration.recip() * dt;
        self.theta = self.theta.clamp(-1.0, 1.0);
        self.up = self.up ^ (self.theta >= 1.0) ^ (self.theta <= -1.0)
    }

    fn render(&mut self, surface: &mut too_renderer::Surface) {
        let (label, _) = &self.gradients[self.pos];
        surface
            .draw(&*self)
            .draw(
                Text::new(label)
                    .fg("#FFF")
                    .bg("#000")
                    .align2(Align2::RIGHT_TOP),
            )
            .draw(
                Text::new(format!("duration: {:.2?}", self.duration))
                    .fg("#FF0")
                    .bg("#000")
                    .align2(Align2::LEFT_TOP),
            )
            .draw(
                Text::new(format!("skew: {:.2?}", self.skew))
                    .fg("#FF0")
                    .bg("#000")
                    .align2(Align2::CENTER_TOP),
            );
    }
}

impl Shape for Demo {
    fn draw(&self, size: too_math::Vec2, mut put: impl FnMut(too_math::Pos2, too_renderer::Pixel)) {
        fn normalize(x: i32, y: i32, w: i32, h: i32, factor: f32) -> f32 {
            let x = x as f32 / (w as f32 - 1.0);
            let y = y as f32 / (h as f32 - 1.0);
            lerp(x, y, factor)
        }

        let (_, gradient) = &self.gradients[self.pos];
        for y in 0..size.y.max(1) {
            for x in 0..size.x {
                let pos = pos2(x, y);
                let t = normalize(x, y, size.x, size.y, self.theta * self.skew);
                let bg = gradient.as_rgba(t);
                put(pos, Pixel::new(' ').bg(bg))
            }
        }
    }
}
