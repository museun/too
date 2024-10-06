use crate::{
    math::{Pos2, Rect, Vec2},
    Cell, Pixel, Rgba, Surface,
};

use super::{
    state::{Debug, LayoutNodes, RenderNodes, ViewId, ViewNodes},
    style::{Stylesheet, Theme},
    Styled,
};

pub struct CroppedSurface<'a> {
    pub rect: Rect,
    pub surface: &'a mut Surface,
}

impl<'a> CroppedSurface<'a> {
    #[inline]
    pub fn from_surface(surface: &'a mut Surface) -> Self {
        Self::new(surface.rect(), surface)
    }

    #[inline]
    pub fn new(rect: Rect, surface: &'a mut Surface) -> Self {
        let rect = surface.rect().intersection(rect);
        Self { rect, surface }
    }

    #[inline]
    pub fn set(&mut self, pos: impl Into<Pos2>, cell: impl Into<Cell>) -> bool {
        let pos = pos.into() + self.rect.left_top();
        if !self.rect.contains(pos) {
            return false;
        }
        self.surface.set(pos, cell);
        true
    }

    pub fn patch(&mut self, pos: impl Into<Pos2>, patch: impl FnOnce(&mut Cell)) {
        let pos = pos.into() + self.rect.left_top();
        if !self.rect.contains(pos) {
            return;
        }

        if let Some(cell) = self.surface.get_mut(pos) {
            patch(cell);
        }
    }

    #[inline]
    pub fn fill(&mut self, bg: impl Into<Rgba>) -> &mut Self {
        self.fill_with(bg.into())
    }

    #[inline]
    pub fn fill_with(&mut self, pixel: impl Into<Pixel>) -> &mut Self {
        self.surface.fill(self.rect, pixel);
        self
    }

    #[inline]
    pub fn fill_rect(&mut self, rect: impl Into<Rect>, bg: impl Into<Rgba>) -> &mut Self {
        let rect = rect.into();
        let rect = rect.translate(self.rect.left_top().to_vec2());
        let rect = rect.intersection(self.rect);
        self.surface.fill(rect, bg.into());
        self
    }

    pub fn fill_rect_with(&mut self, rect: impl Into<Rect>, pixel: impl Into<Pixel>) -> &mut Self {
        let rect = rect.into();
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
    pub nodes: &'a ViewNodes,
    pub layout: &'a LayoutNodes,
    pub render: &'a mut RenderNodes,
    pub stylesheet: &'a mut Stylesheet,
    pub theme: &'a Theme,
    pub surface: CroppedSurface<'b>,
    pub debug: &'a Debug,
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
            self.stylesheet,
            self.theme,
            self.debug,
            id,
            surface,
        );
    }

    pub fn property<T: 'static + Copy>(&mut self, key: Styled<T>) -> T {
        self.stylesheet.get_or_default(key)
    }

    pub fn color(&mut self, key: Styled<Rgba>) -> Rgba {
        self.property(key)
    }

    pub fn rect(&self) -> Rect {
        self.surface.rect()
    }

    pub fn local_rect(&self) -> Rect {
        self.surface.local_rect()
    }

    pub fn debug(&self, msg: impl ToString) {
        self.debug.push(msg);
    }
}
