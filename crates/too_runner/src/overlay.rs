//! Overlays are (optionally) drawn ontop of the surface
//!
//! These can be accessed and configured from the [`Context`](crate::Context)
use std::collections::VecDeque;

use too_layout::{Anchor2, Axis, LinearLayout};
use too_math::vec2;
use too_renderer::{Rgba, SurfaceMut};
use too_shapes::Text;

use crate::{ema_window::WindowStats, App, EmaWindow};

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

/// Allows you to show messages ontop of everything
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
    /// Push a new message into the overlay
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

/// Draws the default fps overlay
///
/// This does not check whether its shown -- thats up to you
pub fn draw_default_fps_overlay(overlay: &mut Overlay, surface: &mut SurfaceMut<'_>) {
    let frame_stats = overlay.fps.get_current_stats();

    let mut alloc = LinearLayout::new(overlay.fps.axis)
        .anchor(overlay.fps.anchor)
        .wrap(true)
        .spacing(vec2(1, 0))
        .layout(surface.rect());

    let (fg, bg) = (overlay.fps.fg, overlay.fps.bg);
    for part in [
        format!("min: {:.2}", frame_stats.min),
        format!("max: {:.2}", frame_stats.max),
        format!("avg: {:.2}", frame_stats.avg),
    ] {
        let part = Text::new(part).fg(fg).bg(bg);
        if let Some(rect) = alloc.allocate(part.size()) {
            surface.crop(rect).draw(part);
        }
    }
}

/// Draws the default debug overlay
///
/// This does not check whether its shown -- thats up to you
pub fn draw_default_debug_overlay(overlay: &mut Overlay, surface: &mut SurfaceMut<'_>) {
    let mut alloc = LinearLayout::new(overlay.debug.axis)
        .anchor(overlay.debug.anchor)
        .wrap(true)
        .spacing(vec2(1, 0))
        .layout(surface.rect());

    let (fg, bg) = (overlay.debug.fg, overlay.debug.bg);

    for msg in overlay.debug.queue.drain(..).rev() {
        let part = Text::new(msg).fg(fg).bg(bg);
        if let Some(rect) = alloc.allocate(part.size()) {
            surface.crop(rect).draw(part);
        }
    }
}

/// A default overlay draw function.
///
/// If you use a custom [`Runner`](crate::Runner) you can use this for [`Runner::post_render`](crate::Runner::post_render)
pub fn draw_default_overlay(_: &mut impl App, overlay: &mut Overlay, mut surface: SurfaceMut<'_>) {
    if overlay.fps.show {
        draw_default_fps_overlay(overlay, &mut surface);
    }

    if overlay.debug.show {
        draw_default_debug_overlay(overlay, &mut surface);
    }
}
