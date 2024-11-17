pub mod debug;

mod state;
pub use state::{debug, Debug, DebugMode, State};

mod response;
pub use response::Response;

mod input;
pub use input::{EventCtx, Handled, InputState, Interest, ViewEvent};

mod ui;
pub use ui::Ui;

mod layout;
pub use layout::{IntrinsicSize, Layer, Layout, LayoutNode, LayoutNodes};

mod render;
pub use render::{CroppedSurface, Render};

mod view_nodes;
pub use view_nodes::{ViewNode, ViewNodes};

mod style;
pub use style::{Elements, Palette, StyleKind};

mod internal_views;

mod adhoc;
pub use adhoc::Adhoc;

mod builder;
pub use builder::{Builder, View, ViewExt};

mod erased;
use erased::Erased;

slotmap::new_key_type! {
    /// An opaque ID for a [`View`](crate::view::View) in the current UI tree
    ///
    /// Nothing is guaranteed about this type.
    pub struct ViewId;
}

pub mod test;

// TODO get rid of this
use crate::math::Size;
#[inline(always)]
#[deprecated(note = "don't use this, use Text when its implemented")]
pub fn measure_text(data: &str) -> Size {
    use unicode_width::UnicodeWidthStr as _;
    Size::new(data.width() as f32, 1.0)
}
