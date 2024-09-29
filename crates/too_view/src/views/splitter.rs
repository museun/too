use too::{anonymous, inverse_lerp, layout::Axis, Pixel};

use crate::{
    geom::{Point, Size, Space},
    view::Context,
    DrawCtx, Event, EventCtx, Handled, Interest, LayoutCtx, Response, UpdateCtx, View, ViewExt,
};

use super::{
    constrain::size,
    list::{column, row},
};

struct Splitter {
    axis: Axis,
    pos: Option<Point>,
    hovered: bool,
}

impl<T: 'static> View<T> for Splitter {
    type Args<'a> = Axis;
    type Response = Option<Point>;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            axis: args,
            pos: None,
            hovered: false,
        }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.axis = args;
        self.pos
    }

    fn interest(&self) -> Interest {
        Interest::MOUSE
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        match event {
            Event::MouseEnter(mouse_move) => self.hovered = true,
            Event::MouseLeave(mouse_move) => self.hovered = false,
            Event::MouseDrag(mouse_drag) => self.pos = Some(mouse_drag.pos),
            _ => return Handled::Bubble,
        };

        Handled::Sink
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        // TODO unpack
        match self.axis {
            Axis::Horizontal => Size::new(1.0, f32::INFINITY),
            Axis::Vertical => Size::new(f32::INFINITY, 1.0),
        }
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        ctx.debug.push(format!("{:?}", self.pos));

        let ch = match self.axis {
            Axis::Horizontal => '┃',
            Axis::Vertical => '━',
        };

        let fg = if self.hovered { "#FF0" } else { "#555" };
        ctx.surface
            .draw(anonymous(|size| |pos| Some(Pixel::new(ch).fg(fg))));
    }
}

pub fn vertical_split<T: 'static, L, R>(
    ctx: &mut Context<T>,
    split_ratio: fn(&mut T) -> &mut f32,
    left: impl FnOnce(&mut Context<T>) -> L,
    right: impl FnOnce(&mut Context<T>) -> R,
) -> Response<(L, R)> {
    split(Axis::Vertical, ctx, split_ratio, left, right)
}

pub fn horizontal_split<T: 'static, L, R>(
    ctx: &mut Context<T>,
    split_ratio: fn(&mut T) -> &mut f32,
    left: impl FnOnce(&mut Context<T>) -> L,
    right: impl FnOnce(&mut Context<T>) -> R,
) -> Response<(L, R)> {
    split(Axis::Horizontal, ctx, split_ratio, left, right)
}

pub fn split<T: 'static, L, R>(
    axis: impl Into<Axis>,
    ctx: &mut Context<T>,
    split_ratio: fn(&mut T) -> &mut f32,
    left: impl FnOnce(&mut Context<T>) -> L,
    right: impl FnOnce(&mut Context<T>) -> R,
) -> Response<(L, R)> {
    let axis = axis.into();
    let rect = ctx.ui.rect;

    let ratio = *split_ratio(ctx.state);
    let (main, cross) = match axis {
        Axis::Horizontal => too::Rect::from(rect).split_horizontal(1, ratio),
        Axis::Vertical => too::Rect::from(rect).split_vertical(1, ratio),
    };

    let (main, cross) = (main.size(), cross.size());

    let show = |ctx: &mut Context<T>| {
        let left = size(main, ctx, left).into_inner();
        let pos = *Splitter::show(axis, ctx);
        let right = size(cross, ctx, right).into_inner();

        let pos = pos.unwrap_or_else(|| rect.center());
        let (x, y, t) = match axis {
            Axis::Horizontal => (rect.left(), rect.right(), pos.x),
            Axis::Vertical => (rect.top(), rect.bottom(), pos.y),
        };
        *(split_ratio)(ctx) = inverse_lerp(x, y, t).unwrap_or(0.5);
        ctx.ui.debug(format!("{x}, {y}, {t}"));

        (left, right)
    };

    let resp = match axis {
        Axis::Horizontal => row(ctx, show),
        Axis::Vertical => column(ctx, show),
    };

    resp.map(|_, inner| (inner, ()))
}
