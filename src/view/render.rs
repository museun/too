use std::{collections::VecDeque, ops::RangeInclusive};

use unicode_segmentation::UnicodeSegmentation as _;

use crate::{
    animation::Animations,
    layout::Axis,
    math::{pos2, Pos2, Rect, Vec2},
    renderer::{Cell, Grapheme, Pixel, Rasterizer, Rgba, Surface, TextShape},
};

use super::{input::InputState, Layer, LayoutNodes, Palette, ViewId, ViewNodes};

/// The render context
pub struct Render<'a, 'b> {
    /// The current view's id
    pub current: ViewId,
    /// Immutable access to the view nodes tree.
    pub nodes: &'a ViewNodes,
    /// Immutable access to the layout nodes tree.
    pub layout: &'a LayoutNodes,
    /// The current palette
    pub palette: &'a Palette,
    /// Mutable access to the animation context
    pub animation: &'a mut Animations,

    pub(super) rect: Rect,
    pub(super) pending: &'a mut VecDeque<ViewId>,
    pub(super) rasterizer: &'b mut dyn Rasterizer,
    pub(super) render: &'a mut RenderNodes,
    pub(super) input: &'a InputState,
}

// TODO determine if this should always be in local space or absolute space
impl<'a, 'b> Render<'a, 'b> {
    /// Draw a specific view
    pub fn draw(&mut self, id: ViewId) {
        self.render.draw(
            id,
            self.nodes,
            self.layout,
            self.input,
            self.palette,
            self.pending,
            self.animation,
            self.rasterizer,
        );
    }

    /// Get the current mouse position
    pub fn mouse_pos(&self) -> Pos2 {
        self.input.mouse_pos()
    }

    /// Get the drawble rect for the view
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Get the offset from the screen rect for this view
    pub fn offset(&self) -> Pos2 {
        self.rect.left_top()
    }

    /// Get the local rect for this view (translated so the origin is your top-left corner)
    pub fn local_rect(&self) -> Rect {
        self.rect.translate(-self.rect.left_top().to_vec2())
    }

    /// Is the current view focused?
    pub fn is_focused(&self) -> bool {
        self.input.is_focused(self.current)
    }

    /// Is the current view hovered?
    pub fn is_hovered(&self) -> bool {
        self.input.is_hovered(self.current)
    }

    /// Is the current view's parent focused?
    pub fn is_parent_focused(&self) -> bool {
        self.input.is_focused(self.nodes.parent())
    }

    /// Is the current view's parent hovered?
    pub fn is_parent_hovered(&self) -> bool {
        self.input.is_hovered(self.nodes.parent())
    }

    /// Get the axis of the parent view
    pub fn parent_axis(&self) -> Axis {
        self.render.current_axis().unwrap()
    }

    /// Shrink the view to this size, giving you a closure to the new render context
    ///
    /// When the closure returns, the size will be reset to the default rect for this view
    pub fn shrink(&mut self, size: impl Into<Vec2>, render: impl FnOnce(&mut Self)) {
        self.crop(self.local_rect().shrink2(size.into()), render)
    }

    /// Scope the render context to the local rect
    ///
    /// When this closure reutrns, the rect will be reset to the default rect for this view
    pub fn local_space(&mut self, render: impl FnOnce(&mut Self)) {
        self.crop(self.local_rect(), render);
    }

    /// Crop this render context to a new rect, giving you a closure to the new render context
    ///
    /// The provided rect cannot exceed the rect given to you by the initial render context.
    ///
    /// The new render context will be localized to the rect. e.g. origin will be the top-left of this new rect
    ///
    /// When the closure returns, the rect will be reset to the default rect for this view
    pub fn crop(&mut self, rect: Rect, render: impl FnOnce(&mut Self)) {
        let old = self.rasterizer.rect();

        let offset = self.rect.left_top().to_vec2();
        let rect = self.rect.intersection(rect.translate(offset));
        self.rasterizer.set_rect(rect);
        render(self);

        self.rasterizer.set_rect(old);
    }

    /// Fill this render context with a specific color
    pub fn fill_bg(&mut self, color: impl Into<Rgba>) -> &mut Self {
        self.rasterizer.fill_bg(color.into());
        self
    }

    /// Fill this render context with a specific pixel
    pub fn fill_with(&mut self, pixel: impl Into<Pixel>) -> &mut Self {
        self.rasterizer.fill_with(pixel.into());
        self
    }

    /// Draw a horizontal line at the `y` offset between `x0..=x1` using the provided pixel
    pub fn horizontal_line(
        &mut self,
        y: i32,
        range: RangeInclusive<i32>,
        pixel: impl Into<Pixel>,
    ) -> &mut Self {
        self.rasterizer.horizontal_line(y, range, pixel.into());
        self
    }

    /// Draw a vertical line at the `x` offset between `y0..=y1` using the provided pixel
    pub fn vertical_line(
        &mut self,
        x: i32,
        range: RangeInclusive<i32>,
        pixel: impl Into<Pixel>,
    ) -> &mut Self {
        self.rasterizer.vertical_line(x, range, pixel.into());
        self
    }

    /// Draws a line in a specific orientation starting an offset `x0,x1..=y0,y1` using the provided pixel
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

    /// Draws a [`TextShape`] into the region
    pub fn text<'t>(&mut self, text: impl Into<TextShape<'t>>) -> &mut Self {
        self.rasterizer.text(text.into());
        self
    }

    /// Update a specific cell.
    ///
    /// This gives you a closure with the cell at that position, if it exists.
    ///
    /// You can use this for changing existing properties on a cell
    pub fn patch(&mut self, pos: impl Into<Pos2>, patch: impl Fn(&mut Cell)) -> &mut Self {
        if let Some(cell) = self.rasterizer.get_mut(pos.into()) {
            patch(cell);
        }
        self
    }

    /// Update the background color of a region
    pub fn patch_bg(&mut self, rect: Rect, color: Rgba) -> &mut Self {
        self.rasterizer.patch_bg(rect, color);
        self
    }

    /// Sets a cell as a specific position
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
    pub(super) current_layer: Layer,
}

impl RenderNodes {
    pub(super) const fn new() -> Self {
        Self {
            axis_stack: Vec::new(),
            current_layer: Layer::Bottom,
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
        pending: &mut VecDeque<ViewId>,
        animation: &mut Animations,
        rasterizer: &mut dyn Rasterizer,
    ) {
        let Some(node) = layout.nodes.get(id) else {
            return;
        };

        if node.rect.width() == 0 || node.rect.height() == 0 {
            return;
        }

        if node.layer > self.current_layer {
            pending.push_back(id);
            return;
        }

        // debug(format_str!("drawing: {id:?}"));

        self.current_layer = node.layer;

        let mut rect = node.rect;
        if let Some(parent) = node.clipped_by {
            let Some(parent) = layout.nodes.get(parent) else {
                return;
            };
            rect = parent.rect.intersection(rect);
            if rect.width() == 0 || rect.height() == 0 {
                return;
            }
        }

        rasterizer.begin(id);
        nodes.begin(id);

        nodes
            .scoped(id, |node| {
                self.axis_stack.push(node.primary_axis());
                rasterizer.set_rect(rect);
                let render = Render {
                    current: id,
                    nodes,
                    layout,
                    palette,
                    animation,
                    rasterizer,
                    input,
                    pending,
                    render: self,
                    rect,
                };
                node.draw(render);
                self.axis_stack.pop();
            })
            .unwrap();

        nodes.end(id);
        rasterizer.end(id);
    }
}

/// A [`Surface`] cropped to a specific [`Rect`]
pub struct CroppedSurface<'a> {
    pub clip_rect: Rect,
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
}

impl<'a> Rasterizer for CroppedSurface<'a> {
    fn set_rect(&mut self, rect: Rect) {
        self.clip_rect = rect;
    }

    fn rect(&self) -> Rect {
        self.clip_rect
    }

    fn patch_bg(&mut self, rect: Rect, color: Rgba) {
        self.surface.patch(rect, |c| c.set_bg(color));
    }

    fn fill_bg(&mut self, color: Rgba) {
        self.surface.fill(self.clip_rect, color);
    }

    fn fill_with(&mut self, pixel: Pixel) {
        self.surface.fill(self.clip_rect, pixel);
    }

    fn line(&mut self, axis: Axis, offset: Pos2, range: RangeInclusive<i32>, pixel: Pixel) {
        let cross: i32 = axis.cross(offset);

        let start: Pos2 = axis.pack(*range.start(), cross);
        let end: Pos2 = axis.pack(*range.end(), cross);

        for y in start.y..=end.y {
            for x in start.x..=end.x {
                self.set(pos2(x, y), pixel);
            }
        }
    }

    fn text(&mut self, shape: TextShape<'_>) {
        for (x, g) in shape.label.graphemes(true).enumerate() {
            let mut cell = Grapheme::new(g).fg(shape.fg).bg(shape.bg);
            if let Some(attr) = shape.attribute {
                cell = cell.attribute(attr)
            }
            self.set(pos2(x as i32, 0), cell);
        }
    }

    fn pixel(&mut self, pos: Pos2, pixel: Pixel) {
        self.set(pos, pixel);
    }

    fn grapheme(&mut self, pos: Pos2, grapheme: Grapheme) {
        self.set(pos, grapheme);
    }

    fn get_mut(&mut self, pos: Pos2) -> Option<&mut Cell> {
        self.get_mut(pos)
    }
}
