use super::{
    cell::{Attribute, Cell, Color},
    Pixel, Renderer,
};

use crate::{
    math::{pos2, rect, Pos2, Rect, Vec2},
    text::MeasureText,
    Event, Text,
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

    #[track_caller]
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
                self.back[index + i] = Cell::Empty // its because its an empty
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

    pub fn fill(&mut self, rect: Rect, pixel: impl Into<Pixel>) -> &mut Self {
        let pixel = pixel.into();
        let rect = self.rect().intersection(rect);
        for y in rect.top()..rect.bottom() {
            for x in rect.left()..rect.right() {
                self.set(pos2(x, y), pixel);
            }
        }
        self
    }

    pub fn text<T: MeasureText>(&mut self, rect: Rect, text: impl Into<Text<T>>) -> &mut Self {
        let text: Text<T> = text.into();
        let rect = self.rect().intersection(rect);
        text.draw(rect, self);
        self
    }

    pub const fn rect(&self) -> Rect {
        rect(self.size)
    }
}

impl Surface {
    pub(crate) fn new(size: Vec2) -> Self {
        Self {
            front: vec![Cell::Empty; size.x as usize * size.y as usize],
            back: vec![Cell::Empty; size.x as usize * size.y as usize],
            size,
        }
    }

    pub(crate) fn resize(&mut self, size: Vec2) {
        if self.size == size {
            return;
        }

        self.front = vec![Cell::Empty; size.x as usize * size.y as usize];
        self.back = vec![Cell::Pixel(Pixel::DEFAULT); size.x as usize * size.y as usize];

        self.size = size;
    }

    // TODO `force invalidate` (rather than a lazy invalidate)
    pub(crate) fn render(&mut self, renderer: &mut impl Renderer) -> std::io::Result<()> {
        let mut state = CursorState::default();
        let mut seen = false;
        let mut wrote_reset = false;
        let mut buf = [0u8; 4];

        for (pos, change) in Self::diff(&mut self.front, &mut self.back, self.size.x) {
            assert!(!change.is_continuation());
            assert!(!change.is_empty());

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
                    use unicode_segmentation::UnicodeSegmentation as _;
                    use unicode_width::UnicodeWidthStr as _;
                    let mut available = self.size.x as usize - pos.x as usize;
                    for cluster in grapheme.cluster.graphemes(true) {
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
        let iter = front.iter_mut().zip(back.iter_mut()).enumerate();
        // TODO clean this up
        iter.filter_map(move |(i, (front, back))| match (&*front, &*back) {
            (Cell::Grapheme(left), Cell::Grapheme(right)) if left.is_different(right) => {
                *front = std::mem::replace(back, Cell::erase());
                Some((Self::index_to_pos(i, width), &*front))
            }
            (Cell::Pixel(left), Cell::Pixel(right)) if left.is_different(right) => {
                *front = std::mem::replace(back, Cell::erase());
                Some((Self::index_to_pos(i, width), &*front))
            }
            (.., Cell::Grapheme(..) | Cell::Pixel(..)) => {
                *front = std::mem::replace(back, Cell::erase());
                Some((Self::index_to_pos(i, width), &*front))
            }
            _ => {
                *front = std::mem::replace(back, Cell::erase());
                None
            }
        })
    }

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
