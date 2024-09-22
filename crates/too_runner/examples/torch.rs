use std::io::Read as _;

use too_crossterm::{Config, Term};
use too_runner::{
    color::Rgba,
    events::{Event, Key},
    math::{lerp, pos2, Pos2, Rect, Vec2},
    pixel::Pixel,
    shapes::{Fill, Shape},
    App, AppRunner, Backend, Context, SurfaceMut,
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

    let term = Term::setup(Config::default())?;
    Demo::new(input.lines()).run(term)
}

enum Mode {
    Torch,
    Focus,
}

struct Demo {
    cursor: Pos2,
    enabled: bool,
    lines: Vec<String>,
    pos: usize,
    mode: Mode,
}

impl Demo {
    const FG: Rgba = Rgba::from_static("#000000");

    fn new(lines: impl IntoIterator<Item = impl ToString>) -> Self {
        let lines: Vec<_> = lines.into_iter().map(|s| s.to_string()).collect();
        Self {
            cursor: Pos2::ZERO,
            enabled: false,
            pos: lines.len(),
            lines,
            mode: Mode::Torch,
        }
    }

    fn scroll_down(&mut self, lines: usize) {
        self.pos = self.pos.saturating_sub(lines);
    }

    fn scroll_up(&mut self, lines: usize) {
        self.pos = (self.pos + lines).min(self.lines.len())
    }

    fn draw_torch(&self, offset: usize, surface: &mut SurfaceMut) {
        const BG: Rgba = Rgba::from_static("#F0E68C");
        const SHADOW: Rgba = Rgba::from_static("#333333");

        fn blend(demo: &Demo, _: Vec2, pos: Pos2) -> Rgba {
            if !demo.enabled {
                return BG;
            }

            let x = (pos.x as f32 - demo.cursor.x as f32) * 1.6;
            let y = (pos.y as f32 - demo.cursor.y as f32) * 3.0;

            let distance = x.hypot(y).sqrt().max(1.5);
            let blend = lerp(0.0, 0.25, distance);

            BG.blend_linear(SHADOW, blend)
        }

        surface
            .draw(Fill::new(if self.enabled { Self::FG } else { BG }))
            .draw(TorchText {
                demo: self,
                offset,
                blend,
            });
    }

    fn draw_focus(&self, offset: usize, surface: &mut SurfaceMut) {
        const SHADOW: Rgba = Rgba::from_static("#AAAAAAAA");
        const BG: Rgba = Rgba::from_static("#111111");

        fn blend(demo: &Demo, size: Vec2, pos: Pos2) -> Rgba {
            if !demo.enabled {
                return BG;
            }

            let rect = Rect::from_center_size(demo.cursor, size / 3);
            if rect.contains(pos) {
                SHADOW
            } else {
                BG
            }
        }

        surface.draw(Fill::new(BG)).draw(TorchText {
            demo: self,
            offset,
            blend,
        });
    }
}

impl App for Demo {
    fn event(&mut self, event: Event, _: Context<'_, impl Backend>, size: Vec2) {
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

        if event.is_keybind_pressed('m') {
            self.mode = match self.mode {
                Mode::Torch => Mode::Focus,
                Mode::Focus => Mode::Torch,
            }
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

    fn render(&mut self, surface: &mut SurfaceMut) {
        let offset = self.lines.len().saturating_sub(self.pos);
        let offset = offset
            .checked_sub(surface.rect().height().saturating_sub_unsigned(1) as _)
            .unwrap_or(offset);

        match self.mode {
            Mode::Torch => self.draw_torch(offset, surface),
            Mode::Focus => self.draw_focus(offset, surface),
        }
    }
}

struct TorchText<'a> {
    demo: &'a Demo,
    offset: usize,
    blend: fn(&'a Demo, Vec2, Pos2) -> Rgba,
}

impl<'a> Shape for TorchText<'a> {
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

                let bg = (self.blend)(self.demo, size, start);
                put(start, Pixel::new(ch).fg(Demo::FG).bg(bg));
                start.x += 1;
            }

            while start.x < size.x {
                let bg = (self.blend)(self.demo, size, start);
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
                let bg = (self.blend)(self.demo, size, start);
                put(pos, Pixel::new(' ').fg(Demo::FG).bg(bg));
            }
        }
    }
}
