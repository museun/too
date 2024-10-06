use core::f32;

use crate::{
    layout::Axis,
    math::{inverse_lerp, Rect},
    view::{
        geom::{Size, Space},
        properties::Elements,
        views::list,
        EventInterest, Id, Layout, Render, Ui, View, ViewId,
    },
    Pixel,
};

#[derive(Debug)]
pub struct Splitter {
    axis: Axis,
}

impl View for Splitter {
    type Args<'v> = Axis;
    type Response = ();

    fn create(axis: Self::Args<'_>, _: &Ui, _: ViewId) -> Self {
        Self { axis }
    }

    fn update(&mut self, args: Self::Args<'_>, _: &Ui, _: ViewId, rect: Rect) -> Self::Response {
        self.axis = args;
    }

    fn event_interests(&self) -> EventInterest {
        EventInterest::MOUSE
    }

    fn layout(&mut self, _: Layout, space: Space) -> Size {
        // TODO axis
        // TODO properties
        let size = match self.axis {
            Axis::Horizontal => Size::new(1.0, f32::INFINITY),
            Axis::Vertical => Size::new(f32::INFINITY, 1.0),
        };
        space.fit(size)
    }

    fn draw(&mut self, mut render: Render) {
        // TODO properties
        let ch = match self.axis {
            Axis::Horizontal => Elements::THICK_VERTICAL_LINE,
            Axis::Vertical => Elements::THICK_HORIZONTAL_LINE,
        };

        let fg = if render.is_hovered() {
            render.theme.info
        } else {
            render.theme.outline
        };
        render.surface.fill_with(Pixel::new(ch).fg(fg));
    }
}

pub fn horizontal_split<L, R, T>(
    ui: &Ui,
    this: &mut T,
    left: impl FnMut(&mut T, &Ui) -> L,
    right: impl FnMut(&mut T, &Ui) -> R,
)
// -> (Response<L>, Response<R>)
where
    L: 'static,
    R: 'static,
{
    split(ui, Axis::Horizontal, this, left, right)
}

pub fn vertical_split<L, R, T>(
    ui: &Ui,
    this: &mut T,
    left: impl FnMut(&mut T, &Ui) -> L,
    right: impl FnMut(&mut T, &Ui) -> R,
)
// -> (Response<L>, Response<R>)
where
    L: 'static,
    R: 'static,
{
    split(ui, Axis::Vertical, this, left, right)
}

pub fn split<L, R, T>(
    ui: &Ui,
    axis: Axis,
    this: &mut T,
    mut left: impl FnMut(&mut T, &Ui) -> L,
    mut right: impl FnMut(&mut T, &Ui) -> R,
)
// -> (Response<L>, Response<R>)
where
    L: 'static,
    R: 'static,
{
    let id = Id::new(ui.current_id()).with("split");
    let ratio = *ui.view_state_mut().get_or(id.with("ratio"), 0.5);
    let rect = ui.current_available_rect();

    let (main, cross) = match axis {
        Axis::Horizontal => rect.split_horizontal(1, ratio),
        Axis::Vertical => rect.split_vertical(1, ratio),
    };

    let show = |ui: &Ui| {
        let (main, cross) = ui.current_children(|ui, children| match children {
            [left, _, right] => (
                ui.size_for_id(*left).unwrap(),
                ui.size_for_id(*right).unwrap(),
            ),
            _ => (Size::from(main.size()), Size::from(cross.size())),
        });

        eprintln!("{main:?} | {cross:?}");

        let left = left(this, ui);

        let resp = ui.mouse_area(|ui| ui.show::<Splitter>(axis));
        let right = right(this, ui);

        if let Some(pos) = resp.dragged().map(|c| c.current) {
            let (x, y, t) = match axis {
                Axis::Horizontal => (rect.left(), rect.right(), pos.x),
                Axis::Vertical => (rect.top(), rect.bottom(), pos.y),
            };
            let (x, y, t) = (x as f32, y as f32, t as f32);
            *ui.view_state_mut().get_or(id.with("ratio"), 0.5) =
                inverse_lerp(x, y, t).unwrap_or(0.5);
        }

        (left, right)
    };

    let resp = list().axis(axis).gap(0).show_children(ui, show);
    // resp.remap_inner()
}
