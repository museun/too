use super::geom::Size;
use unicode_width::UnicodeWidthStr as _;

mod aligned;
pub use aligned::{aligned, Aligned};

mod slider;
pub use slider::{slider, Slider};

mod fill;
pub use fill::{fill, Fill};

mod button;
pub use button::{button, Button, ButtonResponse};

mod layer;
pub use layer::{Layer, Scope};

mod label;
pub use label::{label, Label};

mod list;
pub use list::{list, CrossAlign, Justify, List};

mod wrap;
pub use wrap::{horizontal_wrap, vertical_wrap, Wrap};

mod expander;
pub use expander::{Expander, Separator};

mod mouse_area;
pub use mouse_area::{Dragging, MouseArea, MouseAreaResponse};

mod key_area;
pub use key_area::{key_area, KeyArea, KeyAreaResponse};

mod constrain;
pub use constrain::{Constrain, Unconstrained};

mod progress_bar;
pub use progress_bar::{progress_bar, ProgressBar};

mod toggle_switch;
pub use toggle_switch::{ToggleResponse, ToggleSwitch};

mod border;
pub use border::{border, frame, Border, BorderView};

mod background;
pub use background::Background;

mod margin;
pub use margin::Margin;

mod float;
pub use float::{clip, float, Clip, Float};

mod offset;
pub use offset::Offset;

mod flex;
pub use flex::Flex;

pub mod scrollable;

mod text_input;
pub use text_input::{text_input, TextInput, TextInputResponse};

pub mod drop_down;

// pub mod focus_ring;

// pub mod split_view;

// tree view (why not)
// drop down
// split view
// scrollable
// link (hyperlink support. OSC 8 https://github.com/Alhadis/OSC8-Adoption/)
// panel (docking)
//
// stack (? z-index layering)
//
// enabled (this will show/hide an view)
//
// floating window
//
//
// canvas
// animate
//
//
// rgba | hsva | hsla selector

#[inline(always)]
pub fn measure_text(data: &str) -> Size {
    Size::new(data.width() as f32, 1.0)
}
