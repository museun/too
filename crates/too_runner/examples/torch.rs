use std::io::Read as _;

use too_crossterm::Config;
use too_runner::{
    math::{lerp, pos2, Pos2, Vec2},
    shapes::Fill,
    App, Context, Event, Key, Pixel, Rgba, Shape, Surface,
};

fn main() -> std::io::Result<()> {
    let input = match std::env::args().nth(1).as_deref() {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let term = too_crossterm::setup(Config::default())?;
    too_runner::run(|_| Demo::new(input.lines()), term)
}

struct Demo {
    cursor: Pos2,
    enabled: bool,
    lines: Vec<String>,
    pos: usize,
}

impl Demo {
    const FG: Rgba = Rgba::from_static("#000000");
    const BG: Rgba = Rgba::from_static("#F0E68C");
    const SHADOW: Rgba = Rgba::from_static("#333333");

    fn new(lines: impl IntoIterator<Item = impl ToString>) -> Self {
        let lines: Vec<_> = lines.into_iter().map(|s| s.to_string()).collect();
        Self {
            cursor: Pos2::ZERO,
            enabled: false,
            pos: lines.len(),
            lines,
        }
    }

    fn scroll_down(&mut self, lines: usize) {
        self.pos = self.pos.saturating_sub(lines);
    }

    fn scroll_up(&mut self, lines: usize) {
        self.pos = (self.pos + lines).min(self.lines.len())
    }

    fn maybe_blend(&self, pos: Pos2) -> Rgba {
        if !self.enabled {
            return Self::BG;
        }

        let x = (pos.x as f32 - self.cursor.x as f32) * 1.6;
        let y = (pos.y as f32 - self.cursor.y as f32) * 3.0;

        let distance = x.hypot(y).sqrt().max(1.5);
        let blend = lerp(0.0, 0.25, distance);

        Self::BG.blend_linear(Self::SHADOW, blend)
    }
}

impl App for Demo {
    fn event(&mut self, event: Event, _: Context<'_>, size: Vec2) {
        if event.is_keybind_pressed(' ') {
            self.enabled = !self.enabled
        }

        if event.is_keybind_pressed(Key::PageDown) {
            self.scroll_down(size.y as usize * 2);
        }

        if event.is_keybind_pressed(Key::PageUp) {
            self.scroll_up(size.y as usize * 2);
        }

        if event.is_keybind_pressed(Key::Down) {
            self.scroll_down(1);
        }

        if event.is_keybind_pressed(Key::Up) {
            self.scroll_up(1);
        }

        if let Event::MouseScroll { delta, .. } = event {
            if delta.y.is_negative() {
                self.scroll_up(3)
            } else {
                self.scroll_down(3)
            }
        }

        if let Some(pos) = event.mouse_pos() {
            self.cursor = pos
        }
    }

    fn render(&mut self, surface: &mut Surface) {
        surface.draw(Fill::new(if self.enabled { Self::FG } else { Self::BG }));

        let offset = self.lines.len().saturating_sub(self.pos);
        let offset = offset
            .checked_sub(surface.rect().height().saturating_sub_unsigned(1) as _)
            .unwrap_or(offset);

        surface.draw(ShadowText { demo: self, offset });
    }
}

struct ShadowText<'a> {
    demo: &'a Demo,
    offset: usize,
}

impl<'a> Shape for ShadowText<'a> {
    fn draw(&self, size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
        let mut start = Pos2::ZERO;
        for line in self.demo.lines.iter().skip(self.offset) {
            if start.y >= size.y {
                break;
            }

            for ch in line.chars() {
                if start.x >= size.x {
                    start.x = 0;
                    start.y += 1;
                }
                let bg = self.demo.maybe_blend(start);
                put(start, Pixel::new(ch).fg(Demo::FG).bg(bg));
                start.x += 1;
            }

            while start.x < size.x {
                let bg = self.demo.maybe_blend(start);
                put(start, Pixel::new(' ').fg(Demo::FG).bg(bg));
                start.x += 1;
            }

            start.x = 0;
            start.y += 1;
        }

        if start.y >= size.y {
            return;
        }

        for y in start.y..size.y {
            for x in 0..size.x {
                let pos = pos2(x, y);
                let bg = self.demo.maybe_blend(start);
                put(pos, Pixel::new(' ').fg(Demo::FG).bg(bg));
            }
        }
    }
}
