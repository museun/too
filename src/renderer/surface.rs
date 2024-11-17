use unicode_segmentation::UnicodeSegmentation;

use super::{
    cell::{Attribute, Cell, Color},
    Pixel, Renderer,
};

use crate::{
    backend::Event,
    math::{pos2, rect, Pos2, Rect, Vec2},
};

/// An owned view of a rect region that allows drawing
pub struct Surface {
    front: Vec<Cell>,
    back: Vec<Cell>,
    size: Vec2,
}

impl Surface {
    pub fn update(&mut self, event: &Event) {
        match event {
            &Event::Resize(size) => self.resize(size),
            Event::SwitchAltScreen => Self::reset(&mut self.front, Cell::Empty),
            _ => {}
        }
    }

    pub fn get_mut(&mut self, pos: Pos2) -> Option<&mut Cell> {
        if !self.rect().contains(pos) {
            return None;
        }

        let index = Self::pos_to_index(pos, self.size.x);
        self.back.get_mut(index)
    }

    pub fn set(&mut self, pos: Pos2, cell: impl Into<Cell>) {
        // implictly clip cell
        if !self.rect().contains(pos) {
            return;
        }

        let (width, x) = (self.size.x as usize, pos.x as usize);
        let index = Self::pos_to_index(pos, self.size.x);

        let empty = self
            .back
            .get(index)
            .map(Cell::width)
            .unwrap_or(0)
            .min(width - x);

        // take the old one so we can merge it with the new one
        let mut old = std::mem::take(&mut self.back[index]);

        for i in 0..empty {
            if index + i < self.back.len() {
                self.back[index + i] = Cell::Empty
            }
        }

        let cell = cell.into();
        let width = cell.width().min(width - x);

        Cell::merge(&mut old, cell);
        self.back[index] = old;

        for i in 1..width {
            if index + i < self.back.len() {
                self.back[index + i] = Cell::Continuation
            }
        }
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    #[allow(dead_code)]
    fn set_line(&mut self, line: i32, start: i32, end: i32, pixel: Pixel) {
        let y = Self::pos_to_index(pos2(0, line), self.size.x);
        let (start, end) = (start as usize + y, end as usize + y);
        self.back[start..end].fill(Cell::Pixel(pixel));
    }

    // PERF we can use 'set_line' if we patch any cells afterward
    pub fn fill(&mut self, rect: Rect, pixel: impl Into<Pixel>) {
        let pixel = pixel.into();
        if rect == self.rect() {
            self.back.fill(Cell::Pixel(pixel));
            return;
        }

        // TODO optimize this with line-vectored drawing
        let rect = self.rect().intersection(rect);
        for y in rect.top()..rect.bottom() {
            // self.set_line(y, rect.left(), rect.right(), pixel)
            for x in rect.left()..rect.right() {
                self.set(pos2(x, y), pixel);
            }
        }
    }

    pub const fn rect(&self) -> Rect {
        rect(self.size)
    }
}

impl Surface {
    #[cfg_attr(feature = "profile", profiling::function)]
    pub fn new(size: Vec2) -> Self {
        Self {
            front: vec![Cell::Empty; size.x as usize * size.y as usize],
            back: vec![Cell::Empty; size.x as usize * size.y as usize],
            size,
        }
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    pub fn resize(&mut self, size: Vec2) {
        if self.size == size {
            return;
        }

        let new = size.x as usize * size.y as usize;
        // self.front = vec![Cell::Empty; new];
        // self.back = vec![Cell::Pixel(Pixel::DEFAULT); new];

        self.front.resize(new, Cell::Empty);
        self.front.fill(Cell::Empty);

        self.back.resize(new, Cell::Pixel(Pixel::DEFAULT));
        self.back.fill(Cell::Pixel(Pixel::DEFAULT));

        // let old = self.size.x as usize * self.size.y as usize;
        // let diff = old.saturating_sub(new.abs_diff(old));

        // self.front.resize(new, Cell::Empty);
        // self.back.resize(new, Cell::Pixel(Pixel::DEFAULT));

        // if old != diff {
        //     self.front[..diff].fill(Cell::Empty);
        //     self.back[..diff].fill(Cell::Pixel(Pixel::DEFAULT));
        // }

        self.size = size;
    }

    // TODO `force invalidate` (rather than a lazy invalidate)
    #[cfg_attr(feature = "profile", profiling::function)]
    pub fn render(&mut self, renderer: &mut impl Renderer) -> std::io::Result<()> {
        let mut state = CursorState::default();
        let mut seen = false;
        let mut wrote_reset = false;
        let mut buf = [0u8; 4];

        for (pos, change) in Self::diff(&mut self.front, &mut self.back, self.size.x) {
            if change.is_empty() || change.is_continuation() {
                continue;
            }

            if !seen {
                renderer.begin()?;
                seen = true;
            }

            if state.maybe_move(pos, change.width() as i32) {
                renderer.move_to(pos)?;
            }

            match state.maybe_attr(change.attribute()) {
                Some(attr) if attr == Attribute::RESET => {
                    wrote_reset = true;
                    renderer.reset_attr()?;
                }
                Some(attr) => {
                    wrote_reset = false;
                    renderer.set_attr(attr)?;
                }
                _ => {}
            }

            match state.maybe_fg(change.fg(), wrote_reset) {
                Some(Color::Set(fg)) => renderer.set_fg(fg)?,
                Some(Color::Reset) => renderer.reset_fg()?,
                _ => {}
            }

            match state.maybe_bg(change.bg(), wrote_reset) {
                Some(Color::Set(bg)) => renderer.set_bg(bg)?,
                Some(Color::Reset) => renderer.reset_bg()?,
                _ => {}
            }

            wrote_reset = false;

            match change {
                Cell::Grapheme(grapheme) => {
                    use unicode_width::UnicodeWidthStr as _;
                    let mut available = self.size.x as usize - pos.x as usize;
                    for cluster in UnicodeSegmentation::graphemes(&*grapheme.cluster, true) {
                        match available.checked_sub(cluster.width()) {
                            Some(n) => available = n,
                            None => break,
                        }
                        renderer.write_str(cluster)?;
                    }
                }
                Cell::Pixel(pixel) => {
                    renderer.write_str(pixel.char.encode_utf8(&mut buf))?;
                }
                _ => {}
            }
        }

        if seen {
            if state.maybe_move(Pos2::ZERO, 0) {
                renderer.move_to(Pos2::ZERO)?;
            }
            renderer.reset_bg()?;
            renderer.reset_fg()?;
            renderer.reset_attr()?;
            renderer.end()?;
        }

        Ok(())
    }

    fn diff<'a>(
        front: &'a mut [Cell],
        back: &'a mut [Cell],
        width: i32,
    ) -> impl Iterator<Item = (Pos2, &'a Cell)> {
        front
            .iter_mut()
            .zip(back.iter_mut())
            .enumerate()
            .filter_map(move |(i, (front, back))| {
                if front.is_same(back) {
                    return None;
                }
                *front = back.clone();
                // assert!(!matches!(*front, Cell::Empty));
                Some((Self::index_to_pos(i, width), &*front))
            })
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    fn reset(buf: &mut [Cell], cell: impl Into<Cell>) {
        let cell = cell.into();
        for x in buf {
            *x = cell.clone()
        }
    }

    const fn pos_to_index(pos: Pos2, w: i32) -> usize {
        pos.y as usize * w as usize + pos.x as usize
    }

    const fn index_to_pos(index: usize, w: i32) -> Pos2 {
        let index = index as i32;
        pos2(index % w, index / w)
    }
}

#[derive(Default)]
struct CursorState {
    last: Option<Pos2>,
    fg: Option<Color>,
    bg: Option<Color>,
    attr: Option<Attribute>,
}

impl CursorState {
    fn maybe_move(&mut self, pos: Pos2, width: i32) -> bool {
        let should_move = match self.last {
            Some(last) if last.y != pos.y || last.x != pos.x - width => true,
            None => true,
            _ => false,
        };

        self.last = Some(pos);
        should_move
    }

    fn maybe_fg(&mut self, color: Color, wrote_reset: bool) -> Option<Color> {
        Self::maybe_color(color, wrote_reset, &mut self.fg)
    }

    fn maybe_bg(&mut self, color: Color, wrote_reset: bool) -> Option<Color> {
        Self::maybe_color(color, wrote_reset, &mut self.bg)
    }

    fn maybe_color(color: Color, wrote_reset: bool, cache: &mut Option<Color>) -> Option<Color> {
        if matches!(color, Color::Reuse) {
            return None;
        }

        if wrote_reset {
            cache.replace(color);
            return Some(color);
        }

        match (color, *cache) {
            (Color::Reset, None) => {
                cache.replace(color);
                Some(Color::Reset)
            }
            (Color::Reset, Some(Color::Reset)) => None,
            _ => (cache.replace(color) != Some(color)).then_some(color),
        }
    }

    fn maybe_attr(&mut self, attr: Attribute) -> Option<Attribute> {
        match (attr, self.attr) {
            (a, None) if a == Attribute::RESET => {
                self.attr.replace(attr);
                Some(attr)
            }
            (a, Some(b)) if a == Attribute::RESET && b == Attribute::RESET => None,
            _ => (self.attr.replace(attr) != Some(attr)).then_some(attr),
        }
    }
}
