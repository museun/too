use crate::{
    pixel::{Attribute, Color},
    Buffer, Pixel, Renderer, Shape,
};
use too_events::Event;
use too_math::{Pos2, Rect, Vec2};

pub struct Surface {
    front: Buffer,
    back: Buffer,
}

impl Surface {
    pub fn new(size: Vec2) -> Self {
        Self {
            front: Buffer::new(size),
            back: Buffer::new(size),
        }
    }

    pub fn update(&mut self, event: &Event) {
        match event {
            &Event::Resize(size) => self.resize(size),
            Event::SwitchAltScreen => self.front.reset(),
            _ => {}
        }
    }

    pub fn resize(&mut self, size: Vec2) {
        const DIRTY: Pixel = Pixel {
            char: '!',
            ..Pixel::EMPTY
        };

        self.back.resize(size, Pixel::EMPTY);
        self.front.resize(size, DIRTY);
    }

    pub fn reset(&mut self) {
        self.front.reset();
        self.back.reset();
    }

    pub const fn current(&self) -> &Buffer {
        &self.back
    }

    pub fn current_mut(&mut self) -> &mut Buffer {
        &mut self.back
    }

    pub fn get(&self, pos: Pos2) -> Option<&Pixel> {
        self.current().get(pos)
    }

    pub fn get_mut(&mut self, pos: Pos2) -> Option<&mut Pixel> {
        self.current_mut().get_mut(pos)
    }

    pub fn put(&mut self, pos: Pos2, pixel: Pixel) {
        Self::put_buffer(&mut self.back, pos, pixel)
    }

    #[track_caller]
    pub fn draw(&mut self, shape: impl Shape) -> &mut Self {
        shape.draw(self.rect().size(), |pos, pixel| self[pos].merge(pixel));
        self
    }

    pub fn rect(&self) -> Rect {
        self.current().rect()
    }

    pub fn crop(&mut self, rect: Rect) -> SurfaceMut<'_> {
        SurfaceMut {
            // ensure the new cropped rect is never larger than our current rect
            rect: self.rect().clamp_rect(rect),
            surface: self,
        }
    }

    // TODO `force invalidate` (rather than a lazy invalidate)
    pub fn render(&mut self, renderer: &mut impl Renderer) -> std::io::Result<()> {
        let mut state = CursorState::default();
        let mut seen = false;
        let mut wrote_reset = false;

        for (pos, change) in self.front.diff(&self.back) {
            if !seen {
                renderer.begin()?;
                seen = true;
            }

            if state.maybe_move(pos) {
                renderer.move_to(pos)?;
            }

            match state.maybe_attr(change.attr) {
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

            match state.maybe_fg(change.fg, wrote_reset) {
                Some(Color::Rgba(fg)) => renderer.set_fg(fg)?,
                Some(Color::Reset) => {
                    // PERF if we're going to be writing (a lot of) spaces we don't have to reset the fg until the next visible character
                    renderer.reset_fg()?
                }
                _ => {}
            }

            match state.maybe_bg(change.bg, wrote_reset) {
                Some(Color::Rgba(bg)) => renderer.set_bg(bg)?,
                Some(Color::Reset) => renderer.reset_bg()?,
                _ => {}
            }

            wrote_reset = false;

            renderer.write(change.char)?;
        }

        if seen {
            if state.maybe_move(Pos2::ZERO) {
                renderer.move_to(Pos2::ZERO)?;
            }
            renderer.reset_bg()?;
            renderer.reset_fg()?;
            renderer.reset_attr()?;
            renderer.end()?;
        }

        // this is a forced invalidate. ideally the user knows if we should invalid
        self.back.reset(); // shouldn't we just swap this?
                           // std::mem::swap(&mut self.front, &mut self.back)
        Ok(())
    }

    #[track_caller]
    fn put_buffer(buffer: &mut Buffer, pos: Pos2, pixel: Pixel) {
        if !buffer.contains(pos) {
            return;
        }
        buffer[pos].merge(pixel)
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
    fn maybe_move(&mut self, pos: Pos2) -> bool {
        let should_move = match self.last {
            Some(last) if last.y != pos.y || last.x != pos.x - 1 => true,
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

impl std::ops::Index<Pos2> for Surface {
    type Output = Pixel;
    fn index(&self, index: Pos2) -> &Self::Output {
        &self.current()[index]
    }
}

impl std::ops::IndexMut<Pos2> for Surface {
    fn index_mut(&mut self, index: Pos2) -> &mut Self::Output {
        &mut self.current_mut()[index]
    }
}

/// SurfaceMut is a mutable borrow of a rect, possibly clipped to a specific sub-rect
pub struct SurfaceMut<'a> {
    surface: &'a mut Surface,
    rect: Rect,
}

impl<'a> SurfaceMut<'a> {
    /// The [`Rect`] for this surface
    pub const fn rect(&self) -> Rect {
        self.rect
    }

    /// Crop this [`SurfaceMut`] to a smaller [`Rect`]`
    pub fn crop<'b>(&'b mut self, rect: Rect) -> SurfaceMut<'b>
    where
        'a: 'b,
    {
        self.surface.crop(rect)
    }

    /// Draw this [`Shape`] onto this [`SurfaceMut`]
    ///
    /// This is chainable.
    ///
    /// **Note** Future shapes drawn onto the same surface will be drawn ___ontop___ of prior shapes.
    ///
    /// # Example:
    /// ```rust,no_run
    ///
    /// struct MyShape;
    /// struct Overlay;
    ///
    /// impl Shape for MyShape {
    ///     fn draw(&self, size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
    ///         for y in 0..size.y {
    ///             for x in 0..size.x {
    ///                 put(pos2(x, y), Pixel::char(' ').bg("#F00"))
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// impl Shape for Overlay {
    ///     fn draw(&self, size: Vec2, put: impl FnMut(Pos2, Pixel)) {
    ///         for y in 0..size.y / 2{
    ///             for x in 0..size.x / 2 {
    ///                 put(pos2(x, y), Pixel::char(' ').bg("#0F0"))
    ///             }
    ///         }
    ///     }
    ///     }
    /// }
    ///
    /// // this'll fill the surface with a red background
    /// // then the top-left quarter will be overwritten with a green background
    /// surface.draw(MyShape).draw(Overlay);
    ///
    /// ```
    pub fn draw(&mut self, shape: impl Shape) -> &mut Self {
        shape.draw(self.rect.size(), |pos, pixel| {
            let pos = self.translate(pos);
            // clip any draws outside of this sub-rect
            if !self.rect.contains(pos) {
                return;
            }
            Surface::put_buffer(&mut self.surface.back, pos, pixel)
        });
        self
    }

    /// Put a [`Pixel`] as a [`Pos2`]
    ///
    /// This is chainable.
    ///
    /// If this surface does not contain that position, it does not put it.
    ///
    /// The `pos` here is local to the top-left of this surface's [`Rect`]
    ///
    /// e.g. 0,0 is top-left (the origin) of this rect.
    #[track_caller]
    pub fn put(&mut self, pos: Pos2, pixel: Pixel) -> &mut Self {
        let pos = self.translate(pos);
        // clip any draws outside of this sub-rect
        if !self.rect.contains(pos) {
            return self;
        }
        Surface::put_buffer(&mut self.surface.back, pos, pixel);
        self
    }

    /// Tries to get the [`Pixel`] at this [`Pos2`]
    ///
    /// The `pos` here is local to the top-left of this surface's [`Rect`]
    ///
    /// e.g. 0,0 is top-left (the origin) of this rect.
    pub fn get(&self, pos: Pos2) -> Option<&Pixel> {
        let pos = self.translate(pos);
        self.surface.get(pos)
    }

    /// Tries to get the [`Pixel`], mutably at this [`Pos2`]
    ///
    /// The `pos` here is local to the top-left of this surface's [`Rect`]
    ///
    /// e.g. 0,0 is top-left (the origin) of this rect.
    pub fn get_mut(&mut self, pos: Pos2) -> Option<&mut Pixel> {
        let pos = self.translate(pos);
        self.surface.get_mut(pos)
    }

    fn translate(&self, pos: Pos2) -> Pos2 {
        pos + self.rect.left_top()
    }
}

impl<'a> std::ops::Index<Pos2> for SurfaceMut<'a> {
    type Output = Pixel;
    fn index(&self, index: Pos2) -> &Self::Output {
        let index = self.translate(index);
        &self.surface[index]
    }
}

impl<'a> std::ops::IndexMut<Pos2> for SurfaceMut<'a> {
    fn index_mut(&mut self, index: Pos2) -> &mut Self::Output {
        let index = self.translate(index);
        &mut self.surface[index]
    }
}
