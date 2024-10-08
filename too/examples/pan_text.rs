use std::io::Read as _;

use too::{
    math::{pos2, Pos2},
    App, AppRunner as _, Event,
};
use too_crossterm::{Config, Term};

fn main() -> std::io::Result<()> {
    let input = match std::env::args().nth(1).as_deref() {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let term = Term::setup(Config::default())?;
    Pan::new(input.lines()).run(term)
}

impl App for Pan {
    fn event(&mut self, event: Event, ctx: too::Context<'_>) {
        if let Event::MouseDragHeld { delta, .. } = event {
            self.offset += delta.to_pos2();
        }

        if let Event::MouseScroll {
            delta, modifiers, ..
        } = event
        {
            // does shift mouse scroll not work?
            let scale = if modifiers.is_shift() { 5 } else { 1 };

            if modifiers.is_ctrl() {
                self.offset += pos2(delta.y, delta.x) * scale
            } else {
                self.offset -= delta.to_pos2() * scale
            }
        }

        if event.is_keybind_pressed('r') {
            self.offset = Pos2::ZERO
        }

        if event.is_keybind_pressed('c') {
            self.offset = {
                let center = ctx.size() / 2;
                center.to_pos2() - self.cursor
            }
        }

        if let Some(pos) = event.mouse_pos() {
            self.cursor = pos
        }
    }

    fn render(&mut self, surface: &mut impl too::Canvas, _ctx: too::Context<'_>) {
        let size = surface.rect().size();
        let mut start = self.offset;
        for line in &self.lines {
            if start.y >= size.y {
                break;
            }

            for ch in line.chars() {
                if start.x >= size.x {
                    break;
                }

                surface.set(start, ch);
                start.x += 1;
            }

            start.x = self.offset.x;
            start.y += 1;
        }
    }
}

struct Pan {
    lines: Vec<String>,
    offset: Pos2,
    cursor: Pos2,
}

impl Pan {
    fn new(lines: impl IntoIterator<Item = impl ToString>) -> Self {
        let lines: Vec<_> = lines.into_iter().map(|s| s.to_string()).collect();
        Self {
            lines,
            offset: Pos2::ZERO,
            cursor: Pos2::ZERO,
        }
    }
}
