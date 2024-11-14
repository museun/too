use std::ops::RangeInclusive;

use crate::{
    layout::Axis,
    math::{Pos2, Rect, Vec2},
    rasterizer::{self, Rasterizer, TextShape},
    Animations, Cell, Pixel, Rgba, Surface,
};

use super::{input::InputState, LayoutNodes, Palette, ViewId, ViewNodes};

pub struct CroppedSurface<'a> {
    pub(crate) clip_rect: Rect,
    pub surface: &'a mut Surface,
}

impl<'a> CroppedSurface<'a> {
    pub fn get_mut(&mut self, pos: impl Into<Pos2>) -> Option<&mut Cell> {
        let offset = self.clip_rect.left_top();
        let pos = pos.into() + offset;
        if !self.clip_rect.contains(pos) {
            return None;
        }
        self.surface.get_mut(pos)
    }

    #[inline]
    pub fn set(&mut self, pos: impl Into<Pos2>, cell: impl Into<Cell>) -> bool {
        let offset = self.clip_rect.left_top();
        let pos = pos.into() + offset;
        if !self.clip_rect.contains(pos) {
            return false;
        }
        self.surface.set(pos, cell);
        true
    }

    pub fn patch(&mut self, pos: impl Into<Pos2>, patch: impl FnOnce(&mut Cell)) {
        let offset = self.clip_rect.left_top();
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
}

pub struct Render<'a, 'b> {
    pub current: ViewId,
    pub nodes: &'a ViewNodes,
    pub layout: &'a LayoutNodes,

    pub palette: &'a Palette,
    pub animation: &'a mut Animations,
    pub(super) rasterizer: &'b mut dyn Rasterizer,

    pub(super) rect: Rect,
    pub(super) clip_rect: Rect,

    pub(super) render: &'a mut RenderNodes,
    pub(super) input: &'a InputState,
}

// TODO determine if this should always be in local space or absolute space
impl<'a, 'b> Render<'a, 'b> {
    pub fn draw(&mut self, id: ViewId) {
        self.render.draw(
            id,
            self.nodes,
            self.layout,
            self.input,
            self.palette,
            self.animation,
            self.rasterizer,
            self.rect,
        );
    }

    pub fn current(&self) -> ViewId {
        self.nodes.current()
    }

    pub fn mouse_pos(&self) -> Pos2 {
        self.input.mouse_pos()
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn offset(&self) -> Pos2 {
        self.rect.left_top()
    }

    pub fn local_rect(&self) -> Rect {
        self.rect.translate(-self.rect.left_top().to_vec2())
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

    pub fn shrink(&mut self, size: impl Into<Vec2>, render: impl FnOnce(&mut Self)) {
        self.crop(self.rect.shrink2(size.into()), render)
    }

    pub fn local_space(&mut self, render: impl FnOnce(&mut Self)) {
        self.crop(self.local_rect(), render);
    }

    pub fn crop(&mut self, rect: Rect, render: impl FnOnce(&mut Self)) {
        let old = self.rasterizer.rect();

        let offset = self.rect.left_top().to_vec2();
        let rect = self.rect.intersection(rect.translate(offset));
        self.rasterizer.set_rect(rect);
        render(self);

        self.rasterizer.set_rect(old);
    }

    pub fn fill_bg(&mut self, color: impl Into<Rgba>) -> &mut Self {
        self.rasterizer.fill_bg(color.into());
        self
    }

    pub fn fill_with(&mut self, pixel: impl Into<Pixel>) -> &mut Self {
        self.rasterizer.fill_with(pixel.into());
        self
    }

    pub fn horizontal_line(
        &mut self,
        y: i32,
        range: RangeInclusive<i32>,
        pixel: impl Into<Pixel>,
    ) -> &mut Self {
        self.rasterizer.horizontal_line(y, range, pixel.into());
        self
    }

    pub fn vertical_line(
        &mut self,
        x: i32,
        range: RangeInclusive<i32>,
        pixel: impl Into<Pixel>,
    ) -> &mut Self {
        self.rasterizer.vertical_line(x, range, pixel.into());
        self
    }

    pub fn line(
        &mut self,
        axis: Axis,
        offset: impl Into<Pos2>,
        range: RangeInclusive<i32>,
        pixel: impl Into<Pixel>,
    ) -> &mut Self {
        self.rasterizer
            .line(axis, offset.into(), range, pixel.into());
        self
    }

    pub fn text<'t>(&mut self, text: impl Into<TextShape<'t>>) -> &mut Self {
        self.rasterizer.text(text.into());
        self
    }

    pub fn patch(&mut self, pos: impl Into<Pos2>, patch: impl Fn(&mut Cell)) -> &mut Self {
        if let Some(cell) = self.rasterizer.get_mut(pos.into()) {
            patch(cell);
        }
        self
    }

    pub fn set(&mut self, pos: impl Into<Pos2>, cell: impl Into<Cell>) -> &mut Self {
        let pos = pos.into();
        let cell = cell.into();
        match cell {
            Cell::Grapheme(g) => self.rasterizer.grapheme(pos, g),
            Cell::Pixel(p) => self.rasterizer.pixel(pos, p),
            _ => {}
        }
        self
    }
}

#[derive(Default)]
pub struct RenderNodes {
    axis_stack: Vec<Axis>,
}

impl RenderNodes {
    pub(super) const fn new() -> Self {
        Self {
            axis_stack: Vec::new(),
        }
    }

    pub(super) fn start(&mut self) {
        self.axis_stack.clear();
    }

    pub(super) fn current_axis(&self) -> Option<Axis> {
        self.axis_stack.iter().nth_back(1).copied()
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw(
        &mut self,
        id: ViewId,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        input: &InputState,
        palette: &Palette,
        animation: &mut Animations,
        rasterizer: &mut dyn Rasterizer,
        rect: Rect,
    ) {
        let Some(node) = layout.nodes.get(id) else {
            return;
        };

        let rect = node.rect;
        if rect.width() == 0 || rect.height() == 0 {
            return;
        }

        let mut clip_rect = rect;

        if let Some(parent) = node.clipped_by {
            let Some(parent) = layout.nodes.get(parent) else {
                return;
            };
            // if !rect.partial_contains_rect(parent.rect) {
            //     return;
            // }
            clip_rect = parent.rect.intersection(rect);
        }
        if clip_rect.width() == 0 || clip_rect.height() == 0 {
            return;
        }

        rasterizer.begin(id);
        nodes.begin(id);

        nodes
            .scoped(id, |node| {
                self.axis_stack.push(node.primary_axis());
                // let surface = CroppedSurface {
                //     rect,
                //     clip_rect,
                //     surface: surface.surface,
                // };
                rasterizer.set_rect(clip_rect);
                let render = Render {
                    current: id,
                    nodes,
                    layout,
                    palette,
                    animation,
                    rasterizer,
                    input,
                    render: self,
                    rect,
                    clip_rect,
                };
                node.draw(render);
                self.axis_stack.pop();
            })
            .unwrap();

        nodes.end(id);
        rasterizer.end(id);
    }
}
