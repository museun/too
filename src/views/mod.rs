//! Some premade views, with their builders, styles and responses.
// TODO ensure any builder can be constructed from itself as well
// TODO sort the shorthands

mod aligned;
pub use aligned::{aligned, Aligned};

mod background;
pub use background::Background;

mod border;
pub use border::{border, frame, BorderClass, BorderStyle, Frame};

mod button;
pub use button::{button, Button, ButtonClass, ButtonResponse, ButtonStyle};

mod checkbox;
pub use checkbox::{checkbox, Checkbox, CheckboxClass, CheckboxStyle};

// mod collapsible;
// pub use collapsible::{collapsible, Collapsible, CollapsibleClass, CollapsibleStyle};

mod constrain;
pub use constrain::{Constrain, Unconstrained};

mod expander;
pub use expander::{expander, separator, Expander, Separator, SeparatorClass, SeparatorStyle};

mod fill;
pub use fill::{fill, Fill};

mod flex;
pub use flex::Flexible;

mod key_area;
pub use key_area::{key_area, KeyArea, KeyAreaResponse};

mod label;
pub use label::{label, Label, LabelClass, LabelStyle};

mod list;
pub use list::{list, List, ScrollClass, ScrollStyle};

mod margin;
pub use margin::Padding;

mod mouse_area;
pub use mouse_area::{mouse_area, DraggingResponse, MouseArea, MouseAreaResponse};

mod offset;
pub use offset::Offset;

mod progress;
pub use progress::{progress, Progress, ProgressClass, ProgressStyle};

mod radio;
pub use radio::{radio, Radio, RadioClass, RadioStyle};

mod selected;
pub use selected::{selected, Selected, SelectedClass, SelectedStyle};

mod slider;
pub use slider::{slider, Slider, SliderClass, SliderStyle};

mod text_input;
pub use text_input::{text_input, TextInput, TextInputClass, TextInputResponse, TextInputStyle};

mod todo_value;
pub use todo_value::{todo_value, TodoClass, TodoStyle, TodoValue};

mod toggle;
pub use toggle::{toggle, Toggle};

mod toggle_switch;
pub use toggle_switch::{toggle_switch, ToggleClass, ToggleResponse, ToggleStyle, ToggleSwitch};

mod wrap;
pub use wrap::{horizontal_wrap, vertical_wrap, Wrap};

// pub mod drop_down;
// pub mod scrollable;
// pub mod focus_ring;
// pub mod split_view;

// tree view (why not)
// drop down
// split view
// link (hyperlink support. OSC 8 https://github.com/Alhadis/OSC8-Adoption/)
// panel (docking)
//
// stack (? z-index layering)
//
//
// floating window
//
// canvas
// animate
//
//
// rgba | hsva | hsla selector
