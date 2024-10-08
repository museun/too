use super::{
    geom::{Size, Space},
    LayoutCtx, UpdateCtx, View,
};

pub(crate) struct RootView;
impl<T: 'static> View<T> for RootView {
    type Args<'a> = ();
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self
    }

    fn update(&mut self, _: UpdateCtx<T>, args: Self::Args<'_>) {}

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        ctx.new_layer();
        for &child in ctx.children {
            ctx.compute_layout(child, space);
        }
        space.max
    }
}

mod aligned;
pub use aligned::{
    align, center, center_bottom, center_top, left_bottom, left_center, left_top, right_bottom,
    right_center, right_top,
};

mod background;
pub use background::{background, fill};

mod border;
pub use border::{border, frame, Border, BorderTitle};

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
pub use label::{label, static_label, LabelOptions, LabelParams};

mod list;
pub use list::{column, list, row, CrossAlign, List, MainSpacing};

mod flex;
pub use flex::{expand, flex, Flex};

mod margin;
pub use margin::margin;

mod event_area;
pub use event_area::event_area;

mod mouse_area;
pub use mouse_area::{
    mouse_area, on_click, on_drag, on_scroll, DragMode, Dragged, MouseAreaResponse, MouseEvent,
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

mod canvas;
pub use canvas::canvas;

mod animate;
pub use animate::animate;

mod dark_mode;
pub use dark_mode::toggle_dark_mode;

mod separator;
pub use separator::{horizontal_separator, vertical_separator, Separator};

// float
// text input
// border
// scope / styled thing
