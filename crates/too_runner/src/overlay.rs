use too_layout::{Anchor2, Axis};
use too_renderer::Rgba;

use crate::{ema_window::WindowStats, EmaWindow};

#[derive(Default)] // don't mem swap me
#[non_exhaustive]
/// Overlays are drawn ontop of all over renders
///
/// These are things like an ***FPS*** display
pub struct Overlay {
    /// An FPS overlay
    pub fps: FpsOverlay,
}

/// An FPS overlay
pub struct FpsOverlay {
    /// Should we show this overlay?
    pub show: bool,
    /// What axis should it be rendered on
    pub axis: Axis,
    /// Where should the elements be anchored on the axis
    pub anchor: Anchor2,

    /// Foreground color over the text
    pub fg: Rgba,
    /// Background color over the text
    pub bg: Rgba,

    pub(crate) window: EmaWindow<32>,
}

impl FpsOverlay {
    /// Get the current [`WindowStats`] of the EMA window
    pub fn get_current_stats(&self) -> WindowStats {
        self.window.get()
    }
}

impl Default for FpsOverlay {
    fn default() -> Self {
        Self {
            show: false,
            window: EmaWindow::new(),
            axis: Axis::Horizontal,
            anchor: Anchor2::LEFT_TOP,
            fg: Rgba::from_static("#F00"),
            bg: Rgba::from_static("#000"),
        }
    }
}
