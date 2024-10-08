//! Overlays are (optionally) drawn ontop of the surface
use crate::Surface;

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

impl Overlay {
    /// A default overlay draw function.
    pub fn default_draw(overlay: &mut Overlay, surface: &mut Surface) {
        if overlay.fps.show {
            FpsOverlay::default_draw(overlay, surface);
        }

        if overlay.debug.show {
            DebugOverlay::default_draw(overlay, surface);
        }
    }
}

mod debug_overlay;
pub use debug_overlay::DebugOverlay;

mod fps_overlay;
pub use fps_overlay::FpsOverlay;
