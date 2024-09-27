use std::{collections::VecDeque, default};

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
    /// A Debug message overlay
    pub debug: DebugOverlay,
}

// Allows you to show messages ontop of everything
pub struct DebugOverlay {
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

    /// How many messages to buffer
    pub limit: usize,

    pub(crate) queue: VecDeque<String>,
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self {
            show: false,
            axis: Axis::Vertical,
            anchor: Anchor2::RIGHT_TOP,
            fg: Rgba::from_static("#F00"),
            bg: Rgba::from_static("#000"),
            limit: 100,
            queue: VecDeque::new(),
        }
    }
}

impl DebugOverlay {
    pub fn push(&mut self, msg: impl ToString) {
        while self.queue.len() > self.limit {
            self.queue.pop_back();
        }
        self.queue.push_front(msg.to_string());
    }

    /// Toggle showing this overlay
    pub fn toggle(&mut self) {
        self.show = !self.show
    }
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

    /// Toggle showing this overlay
    pub fn toggle(&mut self) {
        self.show = !self.show
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
