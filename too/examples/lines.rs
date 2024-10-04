use too::{
    math::{pos2, Pos2},
    App, AppRunner, Context, Event, Rgba, Surface,
};
use too_crossterm::{Config, Term};

fn main() -> std::io::Result<()> {
    let term = Term::setup(Config::default())?;
    DrawLines::default().run(term)
}

#[derive(Default)]
struct DrawLines {
    lines: Vec<Line>,
}

impl App for DrawLines {
    fn event(&mut self, event: Event, _ctx: Context<'_>) {
        if event.is_keybind_pressed('r') {
            self.lines.clear();
        }
        if event.is_keybind_pressed('d') {
            self.lines.pop();
        }

        if let Event::MouseDragStart { pos, .. } = event {
            self.lines.push(Line {
                start: pos,
                end: pos,
                color: Rgba::sine(self.lines.len() as f32),
            });
        }

        if let Event::MouseDragHeld { pos, .. } | Event::MouseDragRelease { pos, .. } = event {
            self.lines.last_mut().unwrap().end = pos
        }
    }

    fn render(&mut self, surface: &mut Surface, _ctx: Context<'_>) {
        for line in &self.lines {
            line.draw(surface);
        }
    }
}

struct Line {
    start: Pos2,
    end: Pos2,
    color: Rgba,
}

impl Line {
    fn draw(&self, surface: &mut Surface) {
        let Pos2 { x: sx, y: sy } = self.start;
        let Pos2 { x: ex, y: ey } = self.end;

        let xd = sx.max(ex) - sx.min(ex);
        let yd = sy.max(ey) - sy.min(ey);

        let dx = if sx <= ex { 1 } else { -1 };
        let dy = if sy <= ey { 1 } else { -1 };

        let slope = xd.max(yd);

        for i in 0..=slope {
            let mut x = sx;
            let mut y = sy;
            if xd != 0 {
                x += ((i * xd) / slope) * dx;
            }
            if yd != 0 {
                y += ((i * yd) / slope) * dy;
            }
            surface.set(pos2(x, y), self.color);
        }
    }
}
