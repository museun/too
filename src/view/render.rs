use crate::{
    layout::Axis,
    math::{Pos2, Rect, Vec2},
    AnimationManager, Cell, Pixel, Rgba, Surface,
};

use super::{
    input::InputState,
    state::{LayoutNodes, RenderNodes, ViewId, ViewNodes},
    Palette,
};

pub struct CroppedSurface<'a> {
    pub rect: Rect,
    pub surface: &'a mut Surface,
}

impl<'a> CroppedSurface<'a> {
    pub fn from_surface(surface: &'a mut Surface) -> Self {
        Self::new(surface.rect(), surface)
    }

    pub fn new(rect: Rect, surface: &'a mut Surface) -> Self {
        let rect = surface.rect().intersection(rect);
        Self { rect, surface }
    }

    #[inline]
    pub fn set(&mut self, pos: impl Into<Pos2>, cell: impl Into<Cell>) -> bool {
        let offset = self.rect.left_top();
        if offset.x >= self.rect().right() || offset.y >= self.rect().bottom() {
            return false;
        }

        let pos = pos.into() + offset;
        if !self.rect.contains(pos) {
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
        if !self.rect.contains(pos) {
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
        self.surface.fill(self.rect, pixel);
        self
    }

    pub fn fill_rect(&mut self, rect: impl Into<Rect>, bg: impl Into<Rgba>) -> &mut Self {
        let rect = rect.into();
        // TODO ensure these can't overflow
        let rect = rect.translate(self.rect.left_top().to_vec2());
        let rect = rect.intersection(self.rect);
        self.surface.fill(rect, bg.into());
        self
    }

    pub fn fill_rect_with(&mut self, rect: impl Into<Rect>, pixel: impl Into<Pixel>) -> &mut Self {
        let rect = rect.into();
        // TODO ensure these can't overflow
        let rect = rect.translate(self.rect.left_top().to_vec2());
        let rect = rect.intersection(self.rect);
        self.surface.fill(rect, pixel.into());
        self
    }

    pub fn expand(&mut self, size: impl Into<Vec2>) -> CroppedSurface<'_> {
        CroppedSurface {
            rect: self.rect.expand2(size.into()).intersection(self.rect),
            surface: self.surface,
        }
    }

    pub fn shrink(&mut self, size: impl Into<Vec2>) -> CroppedSurface<'_> {
        self.expand(-size.into())
    }

    pub fn fill_up_to(&mut self, size: impl Into<Vec2>, bg: impl Into<Rgba>) {
        self.fill_up_to_with(size, bg.into())
    }

    pub fn fill_up_to_with(&mut self, size: impl Into<Vec2>, pixel: impl Into<Pixel>) {
        let rect = Rect::from_min_size(self.rect.min, size.into());
        CroppedSurface {
            rect: rect.intersection(self.rect),
            surface: self.surface,
        }
        .fill_with(pixel);
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
    pub(super) layout: &'a LayoutNodes,
    pub(super) render: &'a mut RenderNodes,
    pub palette: &'a Palette,
    pub animation: &'a mut AnimationManager,
    pub surface: CroppedSurface<'b>,
    pub(super) input: &'a InputState,
}

impl<'a, 'b> Render<'a, 'b> {
    pub fn draw(&mut self, id: ViewId) {
        let surface = CroppedSurface {
            rect: self.surface.rect,
            surface: self.surface.surface,
        };
        self.render.draw(
            self.nodes, //
            self.layout,
            self.input,
            self.palette,
            self.animation,
            id,
            surface,
        );
    }

    pub fn current(&self) -> ViewId {
        self.nodes.current()
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
}
