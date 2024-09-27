mod ema_window;
pub use ema_window::{EmaWindow, WindowStats};

pub use too_renderer::{Backend, Command, SurfaceMut};

/// Color types
pub mod color {
    pub use too_renderer::{Gradient, Rgba};
}

/// Pixels are what a Surface consists of
pub mod pixel {
    pub use too_renderer::{Attribute, Color, Pixel};
}

/// Events sent to your application
pub mod events {
    pub use too_events::{Event, Key, Keybind, Modifiers, MouseButton};
}

pub use too_events::EventReader;

/// Layout helpers
pub mod layout {
    pub use too_layout::{Align, Align2, Anchor, Anchor2, Axis, LinearAllocator, LinearLayout};
}

#[doc(inline)]
pub use too_math as math;

/// Shapes are drawable primitives for a Surface
pub mod shapes {
    pub use too_renderer::{anonymous, anonymous_ctx, Shape};
    pub use too_shapes::*;
}

mod app;
pub use app::App;

mod context;
pub use context::Context;

mod app_runner;
pub use app_runner::AppRunner;

mod runner;
pub use runner::Runner;

pub mod overlay;

// Hide this from the docs
// #[cfg(doctests)] doesn't work as expected here
#[doc(hidden)]
pub mod dummy;
