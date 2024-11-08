// TODO redo this whole structure, we should only expose shorthands
// TODO rename all of the response types

mod aligned;
pub use aligned::Aligned;

pub mod slider;
#[doc(inline)]
pub use slider::Slider;

mod fill;
pub use fill::Fill;

pub mod button;
#[doc(inline)]
pub use button::Button;

pub mod checkbox;
#[doc(inline)]
pub use checkbox::Checkbox;

pub mod todo_value;
#[doc(inline)]
pub use todo_value::TodoValue;

pub mod selected;
#[doc(inline)]
pub use selected::Selected;

pub mod radio;
#[doc(inline)]
pub use radio::Radio;

pub mod collapsible;
#[doc(inline)]
pub use collapsible::Collapsible;

mod layer;
pub use layer::{Layer, Scope};

pub mod label;
#[doc(inline)]
pub use label::Label;

pub mod list;
pub use list::{CrossAlign, Justify, List};

mod wrap;
pub use wrap::Wrap;

mod expander;
pub use expander::{Expander, Separator};

mod mouse_area;
pub use mouse_area::{Dragging, MouseArea, MouseAreaResponse};

mod key_area;
pub use key_area::{KeyArea, KeyAreaResponse};

mod constrain;
pub use constrain::{Constrain, Unconstrained};

pub mod progress;
#[doc(inline)]
pub use progress::Progress;

pub mod toggle_switch;
#[doc(inline)]
pub use toggle_switch::{ToggleResponse, ToggleSwitch};

pub mod border;
#[doc(inline)]
pub use border::BorderView;

mod background;
pub use background::Background;

mod margin;
pub use margin::Margin;

mod float;
pub use float::{Clip, Float};

mod offset;
pub use offset::Offset;

mod flex;
pub use flex::Flex;

// pub mod scrollable;

pub mod text_input;
#[doc(inline)]
pub use text_input::{TextInput, TextInputResponse};

// pub mod drop_down;

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
pub fn measure_text(data: &str) -> super::geom::Size {
    use unicode_width::UnicodeWidthStr as _;
    super::geom::Size::new(data.width() as f32, 1.0)
}

pub mod shorthands {
    #[doc(inline)]
    pub use super::aligned::aligned;

    #[doc(inline)]
    pub use super::border::{border, frame};

    #[doc(inline)]
    pub use super::button::button;

    #[doc(inline)]
    pub use super::checkbox::checkbox;

    #[doc(inline)]
    pub use super::collapsible::collapsible;

    #[doc(inline)]
    pub use super::fill::fill;

    #[doc(inline)]
    pub use super::float::{clip, float};

    #[doc(inline)]
    pub use super::key_area::key_area;

    #[doc(inline)]
    pub use super::label::label;

    #[doc(inline)]
    pub use super::list::list;

    #[doc(inline)]
    pub use super::mouse_area::mouse_area;

    #[doc(inline)]
    pub use super::progress::progress;

    #[doc(inline)]
    pub use super::radio::radio;

    #[doc(inline)]
    pub use super::selected::selected;

    #[doc(inline)]
    pub use super::slider::slider;

    #[doc(inline)]
    pub use super::text_input::text_input;

    #[doc(inline)]
    pub use super::todo_value::todo_value;

    #[doc(inline)]
    pub use super::toggle_switch::toggle_switch;

    #[doc(inline)]
    pub use super::wrap::{horizontal_wrap, vertical_wrap};
}
