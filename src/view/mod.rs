pub mod debug;
pub mod helpers;

mod state;
pub(crate) use state::Debug;
pub use state::{debug, DebugMode, State};

mod response;
pub use response::Response;

mod input;
pub use input::{EventCtx, Handled, Interest, ViewEvent};

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
    pub struct ViewId;
}

pub mod test;
