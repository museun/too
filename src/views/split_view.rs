use core::f32;

use crate::{
    layout::Axis,
    math::{inverse_lerp, Rect},
    view::{
        geom::{Flex, Size, Space},
        views::list,
        Builder, Elements, Interest, Layout, Render, Ui, View, ViewExt as _, ViewId,
    },
    Pixel,
};

#[derive(Debug)]
pub struct Splitter {
    axis: Axis,
}

impl<'v> Builder<'v> for Splitter {
    type View = Splitter;
}

impl View for Splitter {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn interests(&self) -> Interest {
        Interest::MOUSE
    }

    fn flex(&self) -> Flex {
        Flex::Tight(1.0)
    }

    fn layout(&mut self, _: Layout, space: Space) -> Size {
        // TODO axis
        // TODO properties
        let size = match self.axis {
            Axis::Horizontal => Size::new(1.0, f32::INFINITY),
            Axis::Vertical => Size::new(f32::INFINITY, 1.0),
        };
        space.constrain_min(size)
    }

    fn draw(&mut self, mut render: Render) {
        // TODO properties
        let ch = match self.axis {
            Axis::Horizontal => Elements::THICK_VERTICAL_LINE,
            Axis::Vertical => Elements::THICK_HORIZONTAL_LINE,
        };

        // let fg = if render.layout.is_hovered() {
        //     render.theme.info
        // } else {
        //     render.theme.outline
        // };
        render
            .surface
            .fill_with(Pixel::new(ch).fg(render.theme.outline));
    }
}

pub fn horizontal_split<L, R>(
    ui: &Ui,
    ratio: &mut f32,
    // this: &mut T,
    left: impl FnMut(&Ui) -> L,
    right: impl FnMut(&Ui) -> R,
)
// -> (Response<L>, Response<R>)
where
    L: 'static,
    R: 'static,
{
    split(ui, Axis::Horizontal, ratio, left, right)
}

pub fn vertical_split<L, R>(
    ui: &Ui,
    ratio: &mut f32,

    left: impl FnMut(&Ui) -> L,
    right: impl FnMut(&Ui) -> R,
)
// -> (Response<L>, Response<R>)
where
    L: 'static,
    R: 'static,
{
    split(ui, Axis::Vertical, ratio, left, right)
}

pub fn split<L, R>(
    ui: &Ui,
    axis: Axis,
    ratio: &mut f32,

    mut left: impl FnMut(&Ui) -> L,
    mut right: impl FnMut(&Ui) -> R,
)
// -> (Response<L>, Response<R>)
where
    L: 'static,
    R: 'static,
{
    let rect = ui.current_available_rect();

    let (main, cross) = match axis {
        Axis::Horizontal => rect.split_horizontal(1, *ratio),
        Axis::Vertical => rect.split_vertical(1, *ratio),
    };

    let show = |ui: &Ui| {
        let (main, cross) = (Size::from(main.size()), Size::from(cross.size()));

        let left = ui.flex(|ui| left(ui));
        let resp = ui
            .mouse_area(|ui| ui.show(Splitter { axis }))
            .flatten_left();
        let right = ui.flex(|ui| right(ui));

        if let Some(pos) = resp.dragged().map(|c| c.current()) {
            let (x, y, t) = match axis {
                Axis::Horizontal => (rect.left(), rect.right(), pos.x),
                Axis::Vertical => (rect.top(), rect.bottom(), pos.y),
            };
            let (x, y, t) = (x as f32, y as f32, t as f32);
            *ratio = inverse_lerp(x, y, t).unwrap_or(0.5);
        }

        (left, right)
    };

    let resp = list().axis(axis).gap(0).show_children(ui, show);
    // resp.remap_inner()
}
