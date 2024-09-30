use crate::{
    geom::{Size, Space},
    LayoutCtx, NoArgs, NoResponse, UpdateCtx, View,
};

mod aligned;
pub use aligned::{align, center};

mod background;
pub use background::background;

mod button;
pub use button::{button, checkbox, radio, selected, todo_value, ButtonParams};

mod constrain;
pub use constrain::{
    constrain, height, max_height, max_size, max_width, min_height, min_size, min_width, size,
    width,
};

mod key_area;
pub use key_area::{hot_key, key_area, key_press, KeyAreaResponse};

mod label;
pub use label::{label, static_label, LabelParams};

mod list;
pub use list::{column, list, row, CrossAlign, ListParams, MainSpacing};

mod margin;
pub use margin::margin;

mod mouse_area;
pub use mouse_area::{
    mouse_area, on_click, on_drag, on_scroll, Dragged, MouseAreaResponse, MouseEvent,
};

mod offset;
pub use offset::offset;

mod progress_bar;
pub use progress_bar::{progress_bar, ProgressBarParams};

mod slider;
pub use slider::{slider, SliderParams};

mod splitter;
pub use splitter::{horizontal_split, split, vertical_split};

mod toggle;
pub use toggle::toggle;

pub(crate) struct RootView;
impl<T: 'static> View<T> for RootView {
    type Args<'a> = NoArgs;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self
    }

    fn update(&mut self, _: UpdateCtx<T>, _: Self::Args<'_>) {}

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        ctx.new_layer();
        for &child in ctx.children {
            ctx.compute_layout(child, space);
        }
        space.max
    }
}

// float
// flex
// constrained
// unconstrained
// text input
// border
// radio
// checkbox (wip)
// todo value
