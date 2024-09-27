use too_crossterm::{Config, Term};

use too_runner::{
    color::Rgba,
    events::{Event, Key},
    layout::{Anchor, Anchor2, Axis, LinearLayout},
    math::{vec2, Vec2},
    shapes::{Fill, Text},
    App, AppRunner, Context, SurfaceMut,
};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;

    Demo {
        elements: Vec::new(),
        axis: Axis::Horizontal,
        anchor: Anchor2 {
            x: Anchor::Min,
            y: Anchor::Min,
        },
        spacing: Vec2::new(1, 1),
        wrap: true,
        mode: Mode::Loose,
    }
    .run(term)
}

enum Mode {
    Tight,
    Loose,
}

struct Demo {
    elements: Vec<Vec2>,
    axis: Axis,
    anchor: Anchor2,
    spacing: Vec2,
    wrap: bool,
    mode: Mode,
}

impl App for Demo {
    fn event(&mut self, event: Event, _ctx: Context<'_>) {
        if event.is_keybind_pressed(Key::Enter) {
            self.elements
                .push(vec2(fastrand::i32(3..=10), fastrand::i32(3..=10)));
        }
        if event.is_keybind_pressed(Key::Backspace) {
            self.elements.pop();
        }
        if event.is_keybind_pressed(' ') {
            self.axis = match self.axis {
                Axis::Horizontal => Axis::Vertical,
                Axis::Vertical => Axis::Horizontal,
            }
        }
        if event.is_keybind_pressed('x') {
            self.anchor.x = match self.anchor.x {
                Anchor::Min => Anchor::Max,
                Anchor::Max => Anchor::Min,
            }
        }
        if event.is_keybind_pressed('y') {
            self.anchor.y = match self.anchor.y {
                Anchor::Min => Anchor::Max,
                Anchor::Max => Anchor::Min,
            }
        }

        if event.is_keybind_pressed('q') {
            self.mode = match self.mode {
                Mode::Tight => Mode::Loose,
                Mode::Loose => Mode::Tight,
            }
        }

        if event.is_keybind_pressed('m') {
            self.wrap = !self.wrap
        }

        if event.is_keybind_pressed('w') {
            self.spacing.y += 1;
            self.spacing.y = self.spacing.y.clamp(0, 5);
        }
        if event.is_keybind_pressed('s') {
            self.spacing.y -= 1;
            self.spacing.y = self.spacing.y.clamp(0, 5);
        }
        if event.is_keybind_pressed('a') {
            self.spacing.x -= 1;
            self.spacing.x = self.spacing.x.clamp(0, 5);
        }
        if event.is_keybind_pressed('d') {
            self.spacing.x += 1;
            self.spacing.x = self.spacing.x.clamp(0, 5);
        }

        if event.is_keybind_pressed('r') {
            self.anchor = Anchor2 {
                x: Anchor::Min,
                y: Anchor::Min,
            };
            self.axis = Axis::Horizontal;
        }
    }

    fn render(&mut self, mut surface: SurfaceMut, _ctx: Context<'_>) {
        let mut layout = LinearLayout::new(self.axis)
            .wrap(self.wrap)
            .spacing(self.spacing)
            .anchor(self.anchor)
            .layout(surface.rect());

        for (i, &size) in self.elements.iter().enumerate() {
            if let Some(rect) = layout.allocate(size) {
                surface
                    .crop(rect)
                    .draw(Fill::new(Rgba::sine(i as f32 * 0.1)));
            }
        }

        let dir = match self.axis {
            Axis::Horizontal => "Horizontal",
            Axis::Vertical => "Vertical",
        };
        let anchor_x = match self.anchor.x {
            Anchor::Min => "Min",
            Anchor::Max => "Max",
        };
        let anchor_y = match self.anchor.y {
            Anchor::Min => "Min",
            Anchor::Max => "Max",
        };
        let elements = self.elements.len();

        surface.draw(
            Text::new(format!(
                "{elements} | {dir}, anchor x: {anchor_x}, anchor y: {anchor_y} | {sx}, {sy} | {wrap}",
                sx = self.spacing.x, sy = self.spacing.y, wrap = self.wrap
            ))
            .fg("#F00")
            .bg("#000"),
        );
    }
}
