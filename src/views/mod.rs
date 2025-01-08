//! Some premade views, with their builders, styles and responses.
// TODO sort the shorthands

mod aligned;
pub use aligned::{aligned, Aligned};

mod background;
pub use background::{background, Background};

mod border;
pub use border::{border, frame, BorderStyle, Frame};

mod button;
pub use button::{button, Button, ButtonResponse, ButtonStyle};

mod checkbox;
pub use checkbox::{checkbox, Checkbox, CheckboxStyle};

mod constrain;
pub use constrain::{Constrain, Unconstrained};

mod expander;
pub use expander::{expander, separator, Expander, Separator, SeparatorStyle};

mod fill;
pub use fill::{fill, Fill};

mod flex;
pub use flex::Flexible;

mod key_area;
pub use key_area::{key_area, KeyArea, KeyAreaResponse};

mod label;
pub use label::{label, Label, LabelStyle};

mod list;
pub use list::{list, List, ScrollStyle};

mod margin;
pub use margin::Padding;

mod mouse_area;
pub use mouse_area::{mouse_area, DraggingResponse, MouseArea, MouseAreaResponse};

mod offset;
pub use offset::Offset;

mod progress;
pub use progress::{progress, Progress, ProgressStyle};

mod radio;
pub use radio::{radio, Radio, RadioStyle};

mod selected;
pub use selected::{selected, Selected, SelectedStyle};

mod slider;
pub use slider::{slider, Slider, SliderStyle};

mod text_input;
pub use text_input::{text_input, TextInput, TextInputResponse, TextInputStyle};

mod todo_value;
pub use todo_value::{todo_value, TodoStyle, TodoValue};

mod toggle;
pub use toggle::{toggle, Toggle};

mod toggle_switch;
pub use toggle_switch::{
    toggle_switch, ToggleResponse, ToggleStyle, ToggleStyleArgs, ToggleSwitch,
};

mod wrap;
pub use wrap::{horizontal_wrap, vertical_wrap, Wrap};
