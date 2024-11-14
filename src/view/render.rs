use std::ops::Range;

use crate::{
    layout::Axis,
    math::{Pos2, Rect, Vec2},
    AnimationManager, Attribute, Cell, Color, Pixel, Rgba, Surface,
};

use super::{
    input::InputState,
    state::{LayoutNodes, RenderNodes, ViewId, ViewNodes},
    Palette,
};

pub struct CroppedSurface<'a> {
    pub rect: Rect,
    pub(super) clip_rect: Rect,
    pub surface: &'a mut Surface,
}

impl<'a> CroppedSurface<'a> {
    pub fn from_surface(surface: &'a mut Surface) -> Self {
        Self::new(surface.rect(), surface.rect(), surface)
    }

    pub fn new(rect: Rect, clip_rect: Rect, surface: &'a mut Surface) -> Self {
        let rect = surface.rect().intersection(rect);
        Self {
            rect,
            clip_rect,
            surface,
        }
    }

    #[inline]
    pub fn set(&mut self, pos: impl Into<Pos2>, cell: impl Into<Cell>) -> bool {
        let offset = self.rect.left_top();
        if offset.x >= self.rect().right() || offset.y >= self.rect().bottom() {
            return false;
        }

        let pos = pos.into() + offset;
        if !self.clip_rect.contains(pos) {
            return false;
        }
        self.surface.set(pos, cell);
        true
    }

    pub fn patch(&mut self, pos: impl Into<Pos2>, patch: impl FnOnce(&mut Cell)) {
        let offset = self.rect.left_top();
        if offset.x >= self.rect().right() || offset.y >= self.rect().bottom() {
            return;
        }

        let pos = pos.into() + offset;
        if !self.clip_rect.contains(pos) {
            return;
        }

        if let Some(cell) = self.surface.get_mut(pos) {
            patch(cell);
        }
    }

    pub fn fill(&mut self, bg: impl Into<Rgba>) -> &mut Self {
        self.fill_with(bg.into())
    }

    pub fn fill_with(&mut self, pixel: impl Into<Pixel>) -> &mut Self {
        self.surface.fill(self.clip_rect, pixel);
        self
    }

    pub fn fill_rect(&mut self, rect: impl Into<Rect>, bg: impl Into<Rgba>) -> &mut Self {
        let rect = rect.into();
        // TODO ensure these can't overflow
        let rect = rect.translate(self.clip_rect.left_top().to_vec2());
        let rect = rect.intersection(self.clip_rect);
        self.surface.fill(rect, bg.into());
        self
    }

    pub fn fill_rect_with(&mut self, rect: impl Into<Rect>, pixel: impl Into<Pixel>) -> &mut Self {
        let rect = rect.into();
        // TODO ensure these can't overflow
        let rect = rect.translate(self.clip_rect.left_top().to_vec2());
        let rect = rect.intersection(self.clip_rect);
        self.surface.fill(rect, pixel.into());
        self
    }

    pub fn expand(&mut self, size: impl Into<Vec2>) -> CroppedSurface<'_> {
        CroppedSurface {
            rect: self.rect.expand2(size.into()).intersection(self.rect),
            clip_rect: self.clip_rect,
            surface: self.surface,
        }
    }

    pub fn shrink(&mut self, size: impl Into<Vec2>) -> CroppedSurface<'_> {
        self.expand(-size.into())
    }

    // crop?
    pub const fn rect(&self) -> Rect {
        self.rect
    }

    pub fn local_rect(&self) -> Rect {
        self.rect.translate(-self.rect.left_top().to_vec2())
    }
}

pub struct Render<'a, 'b> {
    pub current: ViewId,
    pub nodes: &'a ViewNodes,
    pub layout: &'a LayoutNodes,

    pub palette: &'a Palette,
    pub animation: &'a mut AnimationManager,
    pub surface: CroppedSurface<'b>,

    pub(super) render: &'a mut RenderNodes,
    pub(super) input: &'a InputState,
}

impl<'a, 'b> Render<'a, 'b> {
    pub fn draw(&mut self, id: ViewId) {
        self.render.draw(
            self.nodes,
            self.layout,
            self.input,
            self.palette,
            self.animation,
            id,
            CroppedSurface {
                rect: self.surface.rect,
                clip_rect: self.surface.clip_rect,
                surface: self.surface.surface,
            },
        );
    }

    pub fn current(&self) -> ViewId {
        self.nodes.current()
    }

    pub fn mouse_pos(&self) -> Pos2 {
        self.input.mouse_pos()
    }

    pub fn rect(&self) -> Rect {
        self.surface.rect()
    }

    pub fn local_rect(&self) -> Rect {
        self.surface.local_rect()
    }

    pub fn is_focused(&self) -> bool {
        self.input.is_focused(self.current())
    }

    pub fn is_hovered(&self) -> bool {
        self.input.is_hovered(self.current())
    }

    pub fn is_parent_hovered(&self) -> bool {
        self.input.is_hovered(self.nodes.parent())
    }

    pub fn parent_axis(&self) -> Axis {
        self.render.current_axis().unwrap()
    }

    pub fn fill_bg(&mut self, color: impl Into<Rgba>) -> &mut Self {
        todo!();
    }

    pub fn fill_with(&mut self, pixel: impl Into<Pixel>) -> &mut Self {
        todo!();
    }

    pub fn crop(&mut self, rect: Rect) -> &mut Self {
        todo!();
    }

    pub fn shrink_left(&mut self, left: i32) -> &mut Self {
        todo!();
    }

    pub fn shrink_right(&mut self, right: i32) -> &mut Self {
        todo!();
    }

    pub fn shrink_top(&mut self, top: i32) -> &mut Self {
        todo!();
    }

    pub fn shrink_bottom(&mut self, bottom: i32) -> &mut Self {
        todo!();
    }

    pub fn shrink(&mut self, size: Vec2) -> &mut Self {
        todo!();
    }

    pub fn horizontal_line(&mut self, y: i32, range: Range<i32>, pixel: impl Into<Pixel>) {
        todo!();
    }

    pub fn vertical_line(&mut self, x: i32, range: Range<i32>, pixel: impl Into<Pixel>) {
        todo!();
    }

    pub fn line(
        &mut self,
        axis: Axis,
        cross: i32,
        range: Range<i32>,
        pixel: impl Into<Pixel>,
    ) -> &mut Self {
        todo!();
    }

    pub fn text<'t>(&mut self, shape: impl Into<TextShape<'t>>) -> &mut Self {
        todo!();
    }

    pub fn patch(&mut self, pos: Pos2, patch: impl Fn(&mut Cell)) -> &mut Self {
        todo!();
    }

    pub fn pixel(&mut self, pos: Pos2, pixel: impl Into<Pixel>) -> &mut Self {
        todo!();
    }
}

pub struct TextShape<'a> {
    label: &'a str,
    fg: Color,
    bg: Color,
    attribute: Option<Attribute>,
}

impl<'a> From<&'a str> for TextShape<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl<'a> TextShape<'a> {
    pub const fn new(label: &'a str) -> Self {
        Self {
            label,
            fg: Color::Reuse,
            bg: Color::Reset,
            attribute: None,
        }
    }

    pub fn fg(mut self, fg: impl Into<Rgba>) -> Self {
        self.fg = Color::Set(fg.into());
        self
    }

    pub fn bg(mut self, bg: impl Into<Rgba>) -> Self {
        self.bg = Color::Set(bg.into());
        self
    }

    pub fn attribute(mut self, attribute: Attribute) -> Self {
        match &mut self.attribute {
            Some(attr) => *attr |= attribute,
            None => self.attribute = Some(attribute),
        }
        self
    }

    pub fn maybe_attribute(mut self, attribute: Option<Attribute>) -> Self {
        match attribute {
            Some(attr) => self.attribute(attr),
            None => {
                self.attribute.take();
                self
            }
        }
    }
}
